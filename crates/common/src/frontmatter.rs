use crate::dotrain_order::{DotrainOrder, DotrainOrderError};
use dotrain::RainDocument;
pub use rain_metadata::types::authoring::v2::*;
use rain_orderbook_app_settings::{config::ParseConfigSourceError, config_source::ConfigSource};

/// Parse dotrain frontmatter and merges it with top Config if given
pub async fn parse_frontmatter(dotrain: String) -> Result<ConfigSource, ParseConfigSourceError> {
    let frontmatter = RainDocument::get_front_matter(dotrain.as_str()).unwrap_or("");
    Ok(ConfigSource::try_from_string(frontmatter.to_string()).await?)
}

impl DotrainOrder {
    /// Creates a new instance with a clean frontmatter that only includes the
    /// specified deployments and their related fields
    pub async fn new_with_deployments_frontmatter(
        dotrain: String,
        config: Option<String>,
        deployments: &[&str],
    ) -> Result<Self, DotrainOrderError> {
        Self::new(dotrain, config)
            .await?
            .clean_unused_frontmatter_by_deployments(deployments)
            .await
    }

    /// Generates a new instance with a frontmatter that only includes the
    /// specified deployments and their related fields
    pub async fn clean_unused_frontmatter_by_deployments(
        &self,
        deployments: &[&str],
    ) -> Result<Self, DotrainOrderError> {
        // get the original ConfigSource to copy the used fields from
        let frontmatter = RainDocument::get_front_matter(&self.dotrain).unwrap();
        let config_org = match &self.config_content {
            Some(config) => {
                let config_string = ConfigSource::try_from_string(config.clone()).await?;
                let mut frontmatter_config =
                    ConfigSource::try_from_string(frontmatter.to_string()).await?;
                frontmatter_config.merge(config_string)?;
                frontmatter_config
            }
            None => ConfigSource::try_from_string(frontmatter.to_string()).await?,
        };

        // new empty config to copy used field into
        let mut new_config = ConfigSource::default();
        for deployment in deployments {
            // find and insert the specified deployment
            let deployment_ref = self.config.deployments.get(*deployment).ok_or(
                DotrainOrderError::CleanUnusedFrontmatterError(format!(
                    "undefined deployment: {}",
                    deployment
                )),
            )?;
            new_config.deployments.insert(
                deployment.to_string(),
                config_org
                    .deployments
                    .get(*deployment)
                    .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                        "undefined deployment: {}",
                        deployment
                    )))?
                    .clone(),
            );

            // find and insert the deployment scenario
            // make sure to get the root scenario key
            let scenario_main = &deployment_ref.scenario;
            let scenario_key = scenario_main.name.split('.').nth(0).unwrap();
            new_config.scenarios.insert(
                scenario_key.to_string(),
                config_org
                    .scenarios
                    .get(scenario_key)
                    .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                        "undefined scenario: {}",
                        scenario_key
                    )))?
                    .clone(),
            );

            // find and insert charts that reference the scenario
            for (chart_key, chart) in &self.config.charts {
                if chart.scenario.name.split('.').nth(0).unwrap() == scenario_key {
                    new_config.charts.insert(
                        chart_key.clone(),
                        config_org
                            .charts
                            .get(chart_key)
                            .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                                "undefined chart: {}",
                                chart_key
                            )))?
                            .clone(),
                    );
                }
            }

            // find and insert the deployment order
            let (order_key, order) = self
                .config
                .orders
                .iter()
                .find(|(_, v)| *v == &deployment_ref.order)
                .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                    "undefined deployment order: {}",
                    deployment
                )))?;
            new_config.orders.insert(
                order_key.clone(),
                config_org
                    .orders
                    .get(order_key)
                    .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                        "undefined order: {}",
                        order_key
                    )))?
                    .clone(),
            );

            // find and insert the deployment scenario deployer
            let (deployer_key, deployer) = self
                .config
                .deployers
                .iter()
                .find(|(_, v)| *v == &scenario_main.deployer)
                .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                    "undefined scenario deployer: {}",
                    scenario_key
                )))?;
            new_config.deployers.insert(
                deployer_key.clone(),
                config_org
                    .deployers
                    .get(deployer_key)
                    .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                        "undefined deployer: {}",
                        deployer_key
                    )))?
                    .clone(),
            );

            // find and insert the deployment scenario deployer network
            let (network_key, _) = self
                .config
                .networks
                .iter()
                .find(|(_, v)| *v == &deployer.network)
                .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                    "undefined scenario deployer network: {}",
                    scenario_key
                )))?;
            new_config.networks.insert(
                network_key.clone(),
                config_org
                    .networks
                    .get(network_key)
                    .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                        "undefined network: {}",
                        network_key
                    )))?
                    .clone(),
            );

            // find and insert the deployment order network
            let (network_key, _) = self
                .config
                .networks
                .iter()
                .find(|(_, v)| *v == &order.network)
                .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                    "undefined order network: {}",
                    order_key
                )))?;
            new_config.networks.insert(
                network_key.clone(),
                config_org
                    .networks
                    .get(network_key)
                    .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                        "undefined network: {}",
                        network_key
                    )))?
                    .clone(),
            );

            // find and insert the deployment order deployer if it is present
            if let Some(deployer_ref) = &order.deployer {
                let (deployer_key, deployer) = self
                    .config
                    .deployers
                    .iter()
                    .find(|(_, v)| *v == deployer_ref)
                    .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                        "undefined order deployer: {}",
                        order_key
                    )))?;
                new_config.deployers.insert(
                    deployer_key.clone(),
                    config_org
                        .deployers
                        .get(deployer_key)
                        .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                            "undefined order deployer: {}",
                            order_key
                        )))?
                        .clone(),
                );

                // find and insert the deployment order deployer network
                let (network_key, _) = self
                    .config
                    .networks
                    .iter()
                    .find(|(_, v)| *v == &deployer.network)
                    .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                        "undefined order deployer network: {}",
                        order_key
                    )))?;
                new_config.networks.insert(
                    network_key.clone(),
                    config_org
                        .networks
                        .get(network_key)
                        .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                            "undefined network: {}",
                            network_key
                        )))?
                        .clone(),
                );
            }

            // find and insert the deployment order orderbook if it is present
            if let Some(orderbook_ref) = &order.orderbook {
                let (orderbook_key, orderbook) = self
                    .config
                    .orderbooks
                    .iter()
                    .find(|(_, v)| *v == orderbook_ref)
                    .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                        "undefined order orderbook: {}",
                        order_key
                    )))?;
                new_config.orderbooks.insert(
                    orderbook_key.clone(),
                    config_org
                        .orderbooks
                        .get(orderbook_key)
                        .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                            "undefined orderbook: {}",
                            orderbook_key
                        )))?
                        .clone(),
                );

                // find and insert the deployment order orderbook subgraph
                let (sg_key, _) = self
                    .config
                    .subgraphs
                    .iter()
                    .find(|(_, v)| *v == &orderbook_ref.subgraph)
                    .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                        "undefined order orderbook subgraph: {}",
                        order_key
                    )))?;
                new_config.subgraphs.insert(
                    sg_key.clone(),
                    config_org
                        .subgraphs
                        .get(sg_key)
                        .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                            "undefined subgraph: {}",
                            sg_key
                        )))?
                        .clone(),
                );

                // find and insert the deployment order orderbook network
                let (network_key, _) = self
                    .config
                    .networks
                    .iter()
                    .find(|(_, v)| *v == &orderbook.network)
                    .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                        "undefined order orderbook network: {}",
                        order_key
                    )))?;
                new_config.networks.insert(
                    network_key.clone(),
                    config_org
                        .networks
                        .get(network_key)
                        .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                            "undefined network: {}",
                            network_key
                        )))?
                        .clone(),
                );
            }

            // find and insert the deployment order inputs
            for io in &order.inputs {
                let (token_key, token) = self
                    .config
                    .tokens
                    .iter()
                    .find(|(_, v)| *v == &io.token)
                    .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(
                    format!("undefined order input token: {}", order_key),
                ))?;
                new_config.tokens.insert(
                    token_key.clone(),
                    config_org
                        .tokens
                        .get(token_key)
                        .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                            "undefined token: {}",
                            token_key
                        )))?
                        .clone(),
                );

                // find and insert the deployment order input network
                let (network_key, _) = self
                    .config
                    .networks
                    .iter()
                    .find(|(_, v)| *v == &token.network)
                    .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                        "undefined order input token network: {}",
                        token_key
                    )))?;
                new_config.networks.insert(
                    network_key.clone(),
                    config_org
                        .networks
                        .get(network_key)
                        .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                            "undefined network: {}",
                            network_key
                        )))?
                        .clone(),
                );
            }

            // // find and insert the deployment order outputs
            for io in &order.outputs {
                let (token_key, token) = self
                    .config
                    .tokens
                    .iter()
                    .find(|(_, v)| *v == &io.token)
                    .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(
                    format!("undefined order output token: {}", order_key),
                ))?;
                new_config.tokens.insert(
                    token_key.clone(),
                    config_org
                        .tokens
                        .get(token_key)
                        .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                            "undefined token: {}",
                            token_key
                        )))?
                        .clone(),
                );

                // find and insert the deployment order output network
                let (network_key, _) = self
                    .config
                    .networks
                    .iter()
                    .find(|(_, v)| *v == &token.network)
                    .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                        "undefined order output token network: {}",
                        token_key
                    )))?;
                new_config.networks.insert(
                    network_key.clone(),
                    config_org
                        .networks
                        .get(network_key)
                        .ok_or(DotrainOrderError::CleanUnusedFrontmatterError(format!(
                            "undefined network: {}",
                            network_key
                        )))?
                        .clone(),
                );
            }
        }

        // reserialize the new config and construct a new dotrain order with existing dotrain body
        let mut new_dotrain = serde_yaml::to_string(&new_config)
            .map_err(|e| DotrainOrderError::CleanUnusedFrontmatterError(e.to_string()))?;
        new_dotrain.push_str("\n---\n");
        new_dotrain.push_str(self.dotrain.split("---").nth(1).unwrap());

        Self::new(new_dotrain, None).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_to_clean_frontmatter_with_deplyments_happy() {
        let setting = r#"
networks:
    some-network:
        rpc: https://abcd.com
        chain-id: 123
        network-id: 123
        currency: ETH
    n2:
        rpc: https://efgh.com
        chain-id: 44
        network-id: 44
        currency: RRR

subgraphs:
    some-sg: https://www.some-sg.com
    sg2: https://www.sg2.com

deployers:
    some-deployer:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd2B5Ba
    d2:
        network: some-network
        address: 0xF14E09601A47552De6aBd3A0B165607FaFd22134

orderbooks:
    some-orderbook:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: some-network
        subgraph: some-sg
    ob2:
        address: 0xc95A5f8eFe14d7a20BD2E5BAFEC4E71f8Ce0B9A6
        network: n2
        subgraph: sg2
"#;
        let dotrain = r#"
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
    token3:
        network: some-network
        address: 0x8f3cf7ad23cd3cadbd9735aff958023239c6a063
        decimals: 77
        label: T3
        symbol: T3

scenarios:
    some-scenario:
        network: some-network
        deployer: some-deployer
        scenarios: 
            child-scenario:
                bindings:
                    key1: value1
    s2:
        network: some-network
        deployer: d2
        scenarios: 
            child-scenario:
                bindings:
                    key1: value1
                    key2: value2

orders:
    some-order:
        inputs:
            - token: token1
              vault-id: 1
        outputs:
            - token: token1
              vault-id: 1
        deployer: some-deployer
        orderbook: some-orderbook

deployments:
    some-deployment:
        scenario: some-scenario.child-scenario
        order: some-order

charts:
    some-chart:
        scenario: some-scenario
        metrics:
            - label: A metric
              description: A description
              unit-prefix: $
              unit-suffix: USD
              value: 0.1
        plots:
            plot1:
                title: "My plot"
                subtitle: "My subtitle"
                marks:
                    - type: dot
                      options:
                        x: "0.1"
                        y: "0.2"
                        stroke: "black"
    other-chart:
        scenario: s2
        metrics:
            - label: B metric
              description: B description
              unit-prefix: $
              unit-suffix: EUR
              value: 0.1
    another-chart:
        scenario: s2.child-scenario
        metrics:
            - label: A metric
              description: A description
              unit-prefix: $
              unit-suffix: USD
              value: 0.1
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;"#;

        let result = DotrainOrder::new_with_deployments_frontmatter(
            dotrain.to_string(),
            Some(setting.to_string()),
            &["some-deployment"],
        )
        .await
        .unwrap();

        let expected_dotrain = r#"networks:
  some-network:
    rpc: https://abcd.com/
    chain-id: 123
    network-id: 123
    currency: ETH
subgraphs:
  some-sg: https://www.some-sg.com/
orderbooks:
  some-orderbook:
    address: 0xc95a5f8efe14d7a20bd2e5bafec4e71f8ce0b9a6
    network: some-network
    subgraph: some-sg
tokens:
  token1:
    network: some-network
    address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
    decimals: 6
    label: T1
    symbol: T1
deployers:
  some-deployer:
    address: 0xf14e09601a47552de6abd3a0b165607fafd2b5ba
    network: some-network
orders:
  some-order:
    inputs:
    - token: token1
      vault-id: '0x1'
    outputs:
    - token: token1
      vault-id: '0x1'
    deployer: some-deployer
    orderbook: some-orderbook
scenarios:
  some-scenario:
    deployer: some-deployer
    scenarios:
      child-scenario:
        bindings:
          key1: value1
charts:
  some-chart:
    scenario: some-scenario
    plots:
      plot1:
        title: My plot
        subtitle: My subtitle
        marks:
        - type: dot
          options:
            x: '0.1'
            y: '0.2'
            stroke: black
    metrics:
    - label: A metric
      description: A description
      unit-prefix: $
      unit-suffix: USD
      value: '0.1'
deployments:
  some-deployment:
    scenario: some-scenario.child-scenario
    order: some-order

---

#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;"#;

        let expected = DotrainOrder::new(expected_dotrain.to_string(), None)
            .await
            .unwrap();

        assert_eq!(result.config, expected.config);
        assert_eq!(result.dotrain, expected.dotrain);
    }

    #[tokio::test]
    async fn test_to_clean_frontmatter_with_deplyments_unhappy() {
        let setting = r#"
networks:
    some-network:
        rpc: https://abcd.com
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
"#;
        let dotrain = r#"
tokens:
    token1:
        network: some-network
        address: 0xc2132d05d31c914a87c6611c10748aeb04b58e8f
        decimals: 6
        label: T1
        symbol: T1

scenarios:
    some-scenario:
        network: some-network
        deployer: some-deployer
        scenarios: 
            child-scenario:
                bindings:
                    key1: value1

orders:
    some-order:
        inputs:
            - token: token1
              vault-id: 1
        outputs:
            - token: token1
              vault-id: 1
        deployer: some-deployer

deployments:
    some-deployment:
        scenario: some-scenario.child-scenario
        order: some-order
---
#calculate-io
_ _: 0 0;
#handle-io
:;
#handle-add-order
:;"#;

        let result = DotrainOrder::new_with_deployments_frontmatter(
            dotrain.to_string(),
            Some(setting.to_string()),
            &["some-other-deployment"],
        )
        .await;

        matches!(
            result,
            Err(DotrainOrderError::CleanUnusedFrontmatterError(_))
        );
    }
}
