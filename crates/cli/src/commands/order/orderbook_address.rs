use crate::execute::Execute;
use crate::output::{output, SupportedOutputEncoding};
use anyhow::{anyhow, Result};
use clap::Parser;
use rain_orderbook_app_settings::Config;
use rain_orderbook_common::dotrain_order::DotrainOrder;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Parser, Clone)]
pub struct OrderbookAddress {
    #[arg(
        short = 'f',
        long,
        help = "Path to the .rain file specifying the order"
    )]
    dotrain_file: PathBuf,

    #[arg(short = 'c', long, help = "Path to the settings yaml file")]
    settings_file: Option<PathBuf>,

    #[arg(short = 'e', long, help = "Deployment key to select from frontmatter")]
    deployment: String,

    #[arg(short = 'o', long, help = "Output encoding", default_value = "binary")]
    encoding: SupportedOutputEncoding,
}

impl Execute for OrderbookAddress {
    async fn execute(&self) -> Result<()> {
        let dotrain = read_to_string(self.dotrain_file.clone()).map_err(|e| anyhow!(e))?;
        let settings = match &self.settings_file {
            Some(settings_file) => {
                Some(read_to_string(settings_file.clone()).map_err(|e| anyhow!(e))?)
            }
            None => None,
        };
        let order = DotrainOrder::new(dotrain, settings).await?;
        let order_config: Config = order.clone().config;

        let config_deployment = order_config
            .deployments
            .get(&self.deployment)
            .ok_or(anyhow!("specified deployment is undefined!"))?;

        let network_name = config_deployment.scenario.deployer.network.name.clone();
        println!("{}", network_name);

        let orderbook = order_config
            .orderbooks
            .iter()
            .find(|(k, v)| **k == network_name || v.network.name == network_name)
            .ok_or(anyhow!("specified orderbook is undefined!"))?
            .1;

        let orderbook_address = orderbook.address.to_vec();
        output(&None, self.encoding.clone(), &orderbook_address)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use std::str::FromStr;

    #[test]
    fn verify_cli() {
        OrderbookAddress::command().debug_assert();
    }

    #[test]
    fn test_cli_args() {
        let dotrain_file = PathBuf::from_str("./some/dotrain_file.dotrain").unwrap();
        let settings_file = PathBuf::from_str("./some/settings_file.dotrain").unwrap();
        let deployment_str = "some-deployment";
        let output_str = "hex";

        let cmd = OrderbookAddress::command();
        let result = cmd
            .try_get_matches_from(vec![
                "cmd",
                "-f",
                dotrain_file.to_str().unwrap(),
                "-c",
                settings_file.to_str().unwrap(),
                "-e",
                deployment_str,
                "-o",
                output_str,
            ])
            .unwrap();
        assert_eq!(
            result.get_one::<PathBuf>("dotrain_file"),
            Some(&dotrain_file)
        );
        assert_eq!(
            result.get_one::<PathBuf>("settings_file"),
            Some(&settings_file)
        );
        assert_eq!(
            result.get_one::<String>("deployment"),
            Some(&deployment_str.to_string())
        );
        assert_eq!(
            result.get_one::<SupportedOutputEncoding>("encoding"),
            Some(&SupportedOutputEncoding::Hex)
        );
    }

    #[tokio::test]
    async fn test_execute_same_name() {
        let dotrain = "
networks:
    some-network:
        rpc: https://some-url.com
        chain-id: 123
        network-id: 123
        currency: ETH

subgraphs:
    some-sg: https://www.some-sg.com

deployers:
    some-deployer:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba

orderbooks:
    some-orderbook:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: some-network
        subgraph: some-sg

tokens:
    token1:
        network: some-network
        address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
        decimals: 6
        label: T1
        symbol: T1
    token2:
        network: some-network
        address: 0x8f3cf7ad23cd3cadbd9735aff958023239c6a063
        decimals: 18
        label: T2
        symbol: T2

scenarios:
    some-scenario:
        network: some-network
        deployer: some-deployer

orders:
    some-order:
        inputs:
            - token: token1
              vault-id: 1
        outputs:
            - token: token2
              vault-id: 1
        deployer: some-deployer
        orderbook: some-orderbook

deployments:
    some-deployment:
        scenario: some-scenario
        order: some-order
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#post-add-order
:;";

        let dotrain_path = "./test_dotrain.dotrain";
        let _ = std::fs::write(dotrain_path, dotrain);

        let orderbook_adress = OrderbookAddress {
            dotrain_file: dotrain_path.into(),
            settings_file: None,
            deployment: "some-deployment".to_string(),
            encoding: SupportedOutputEncoding::Hex,
        };
        // should succeed without err
        orderbook_adress.execute().await.unwrap();

        // remove test file
        std::fs::remove_file(dotrain_path).unwrap();
    }

    #[tokio::test]
    async fn test_execute_diff_name() {
        let dotrain = "
networks:
    some-network:
        rpc: https://some-url.com
        chain-id: 123
        network-id: 123
        currency: ETH

subgraphs:
    some-sg: https://www.some-sg.com

deployers:
    some-deployer:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba

orderbooks:
    some-network:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: some-network
        subgraph: some-sg

tokens:
    token1:
        network: some-network
        address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
        decimals: 6
        label: T1
        symbol: T1
    token2:
        network: some-network
        address: 0x8f3cf7ad23cd3cadbd9735aff958023239c6a063
        decimals: 18
        label: T2
        symbol: T2

scenarios:
    some-scenario:
        network: some-network
        deployer: some-deployer

orders:
    some-order:
        inputs:
            - token: token1
              vault-id: 1
        outputs:
            - token: token2
              vault-id: 1
        deployer: some-deployer
        orderbook: some-network

deployments:
    some-deployment:
        scenario: some-scenario
        order: some-order
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#post-add-order
:;";

        let dotrain_path = "./test_dotrain.dotrain";
        let _ = std::fs::write(dotrain_path, dotrain);

        let orderbook_adress = OrderbookAddress {
            dotrain_file: dotrain_path.into(),
            settings_file: None,
            deployment: "some-deployment".to_string(),
            encoding: SupportedOutputEncoding::Hex,
        };
        // should succeed without err
        orderbook_adress.execute().await.unwrap();

        // remove test file
        std::fs::remove_file(dotrain_path).unwrap();
    }
}