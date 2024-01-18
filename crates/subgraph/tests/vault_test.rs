use cynic::Id;
use rain_orderbook_subgraph_queries::types::vault::{VaultQuery, VaultQueryVariables};

#[test]
fn vaults_query_gql_output() {
    use cynic::QueryBuilder;

    let id = Id::new("1234");
    let request_body = VaultQuery::build(VaultQueryVariables { id: &id });

    let expected_query = "query VaultQuery($id: ID!) {
  vault(id: $id) {
    id
    owner {
      id
    }
    deposits {
      id
      sender {
        id
      }
      amount
      amountDisplay
    }
    withdraws {
      id
      sender {
        id
      }
      amount
      amountDisplay
      requestedAmount
      requestedAmountDisplay
    }
  }
}

";
    assert_eq!(request_body.query, expected_query);
}
