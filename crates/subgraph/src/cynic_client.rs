use cynic::{
    serde::{Deserialize, Serialize},
    GraphQlError, GraphQlResponse, QueryBuilder, QueryFragment,
};
use reqwest::Url;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CynicClientError {
    #[error("Graphql errors: {}", .0.iter().map(|e| e.message.clone()).collect::<Vec<String>>().join(", "))]
    GraphqlError(Vec<GraphQlError>),
    #[error("Subgraph query returned no data")]
    Empty,
    #[error("Request Error: {0}")]
    Request(#[from] reqwest::Error),
}

pub trait CynicClient {
    async fn query<R: QueryFragment + QueryBuilder<V> + for<'a> Deserialize<'a>, V: Serialize>(
        &self,
        base_url: Url,
        variables: V,
    ) -> Result<R, CynicClientError> {
        let request_body = R::build(variables);

        let response = reqwest::Client::new()
            .post(base_url.clone())
            .json(&request_body)
            .send()
            .await.map_err(|e| CynicClientError::Request(e))?;

        let response_deserialized: GraphQlResponse<R> =
            response.json::<GraphQlResponse<R>>().await.map_err(|e| CynicClientError::Request(e))?;

        match response_deserialized.errors {
            Some(errors) => Err(CynicClientError::GraphqlError(errors)),
            None => response_deserialized
                .data
                .ok_or(CynicClientError::Empty),
        }
    }
}
