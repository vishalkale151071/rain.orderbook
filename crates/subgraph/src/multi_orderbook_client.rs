use futures::future::join_all;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tsify::Tsify;

use crate::{
    types::common::{
        OrderWithSubgraphName, OrdersListFilterArgs, VaultWithSubgraphName, VaultsListFilterArgs,
    },
    OrderbookSubgraphClient, OrderbookSubgraphClientError, PaginationArgs,
};

#[cfg(target_family = "wasm")]
use rain_orderbook_bindings::impl_wasm_traits;
#[cfg(target_family = "wasm")]
use serde_wasm_bindgen::{from_value, to_value};
#[cfg(target_family = "wasm")]
use wasm_bindgen::convert::{
    js_value_vector_from_abi, js_value_vector_into_abi, FromWasmAbi, IntoWasmAbi,
    LongRefFromWasmAbi, RefFromWasmAbi, TryFromJsValue, VectorFromWasmAbi, VectorIntoWasmAbi,
};
#[cfg(target_family = "wasm")]
use wasm_bindgen::describe::{inform, WasmDescribe, WasmDescribeVector, VECTOR};
#[cfg(target_family = "wasm")]
use wasm_bindgen::{JsValue, UnwrapThrowExt};

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct MultiSubgraphArgs {
    #[tsify(type = "string")]
    url: Url,
    name: String,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(MultiSubgraphArgs);

pub struct MultiOrderbookSubgraphClient {
    subgraphs: Vec<MultiSubgraphArgs>,
}
impl MultiOrderbookSubgraphClient {
    pub fn new(subgraphs: Vec<MultiSubgraphArgs>) -> Self {
        Self { subgraphs }
    }

    fn get_orderbook_subgraph_client(&self, url: Url) -> OrderbookSubgraphClient {
        OrderbookSubgraphClient::new(url)
    }

    pub async fn orders_list(
        &self,
        filter_args: OrdersListFilterArgs,
        pagination_args: PaginationArgs,
    ) -> Result<Vec<OrderWithSubgraphName>, OrderbookSubgraphClientError> {
        let futures = self.subgraphs.iter().map(|subgraph| {
            let url = subgraph.url.clone();
            let filter_args = filter_args.clone();
            let pagination_args = pagination_args.clone();
            async move {
                let client = self.get_orderbook_subgraph_client(url);
                let orders = client.orders_list(filter_args, pagination_args).await?;
                let wrapped_orders: Vec<OrderWithSubgraphName> = orders
                    .into_iter()
                    .map(|order| OrderWithSubgraphName {
                        order,
                        subgraph_name: subgraph.name.clone(),
                    })
                    .collect();
                Ok::<_, OrderbookSubgraphClientError>(wrapped_orders)
            }
        });

        let results = join_all(futures).await;

        let mut all_orders: Vec<OrderWithSubgraphName> = results
            .into_iter()
            .filter_map(Result::ok)
            .flatten()
            .collect();

        all_orders.sort_by(|a, b| {
            let a_timestamp = a.order.timestamp_added.0.parse::<i64>().unwrap_or(0);
            let b_timestamp = b.order.timestamp_added.0.parse::<i64>().unwrap_or(0);
            b_timestamp.cmp(&a_timestamp)
        });

        Ok(all_orders)
    }

    pub async fn vaults_list(
        &self,
        filter_args: VaultsListFilterArgs,
        pagination_args: PaginationArgs,
    ) -> Result<Vec<VaultWithSubgraphName>, OrderbookSubgraphClientError> {
        let futures = self.subgraphs.iter().map(|subgraph| {
            let url = subgraph.url.clone();
            let filter_args = filter_args.clone();
            let pagination_args = pagination_args.clone();
            async move {
                let client = self.get_orderbook_subgraph_client(url);
                let vaults = client.vaults_list(filter_args, pagination_args).await?;
                let wrapped_vaults: Vec<VaultWithSubgraphName> = vaults
                    .into_iter()
                    .map(|vault| VaultWithSubgraphName {
                        vault,
                        subgraph_name: subgraph.name.clone(),
                    })
                    .collect();
                Ok::<_, OrderbookSubgraphClientError>(wrapped_vaults)
            }
        });

        let results = join_all(futures).await;

        let all_vaults: Vec<VaultWithSubgraphName> = results
            .into_iter()
            .filter_map(Result::ok)
            .flatten()
            .collect();

        Ok(all_vaults)
    }
}
