mod add;
mod calldata;
mod cleanup;
mod compose;
mod detail;
mod list;
mod orderbook_address;
mod remove;

use crate::commands::order::orderbook_address::OrderbookAddress;
use crate::execute::Execute;
use add::CliOrderAddArgs;
use anyhow::Result;
use calldata::AddOrderCalldata;
use clap::Parser;
use cleanup::Cleanup;
use compose::Compose;

use detail::CliOrderDetailArgs;
use list::CliOrderListArgs;
use remove::CliOrderRemoveArgs;

#[derive(Parser)]
pub enum Order {
    #[command(about = "List all Orders", alias = "ls")]
    List(CliOrderListArgs),

    #[command(about = "View an Order", alias = "view")]
    Detail(CliOrderDetailArgs),

    #[command(about = "Create an Order", alias = "add")]
    Create(CliOrderAddArgs),

    #[command(about = "Remove an Order", alias = "rm")]
    Remove(CliOrderRemoveArgs),

    #[command(about = "Compose a .rain order file to Rainlang", alias = "comp")]
    Compose(Compose),

    #[command(
        about = "Generate calldata for addOrder from a composition",
        alias = "call"
    )]
    Calldata(AddOrderCalldata),

    #[command(
        about = "Get the orderbook address for a given order",
        alias = "ob-addr"
    )]
    OrderbookAddress(OrderbookAddress),

    #[command()]
    Cleanup(Cleanup),
}

impl Execute for Order {
    async fn execute(&self) -> Result<()> {
        match self {
            Order::List(list) => list.execute().await,
            Order::Detail(detail) => detail.execute().await,
            Order::Create(create) => create.execute().await,
            Order::Remove(remove) => remove.execute().await,
            Order::Compose(compose) => compose.execute().await,
            Order::Calldata(calldata) => calldata.execute().await,
            Order::OrderbookAddress(orderbook_address) => orderbook_address.execute().await,
            Order::Cleanup(cleanup) => cleanup.execute().await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_command() {
        Order::command().debug_assert();
    }
}
