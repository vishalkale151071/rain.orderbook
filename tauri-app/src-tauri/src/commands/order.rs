use rain_orderbook_common::{
    subgraph::SubgraphArgs
};
use rain_orderbook_subgraph_queries::types::{
    order::Order as OrderDetail,
};

#[tauri::command]
pub async fn orders_list(subgraph_args: SubgraphArgs) -> Result<Vec<OrdersListItem>, String> {
    subgraph_args
        .to_subgraph_client()
        .await
        .map_err(|_| String::from("Subgraph URL is invalid"))?
        .orders()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn order_detail(id: String, subgraph_args: SubgraphArgs) -> Result<OrderDetail, String> {
    subgraph_args
        .to_subgraph_client()
        .await
        .map_err(|_| String::from("Subgraph URL is invalid"))?
        .order(id.into())
        .await
        .map_err(|e| e.to_string())
}
