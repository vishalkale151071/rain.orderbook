use anyhow::Result;
use clap::command;
use clap::{Parser, Subcommand};

pub mod addorder;
pub mod deposit;
pub mod listorders;
pub mod registry;
pub mod removeorder;
pub mod withdraw;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    orderbook: Orderbook,
}

#[derive(Subcommand)]
pub enum Orderbook {
    /// Deposit tokens into then vault
    Deposit(deposit::Deposit),

    /// Withdraw Tokens from vault
    Withdraw(withdraw::Withdraw),

    /// Add order to orderbook
    AddOrder(addorder::AddOrder),

    /// Remove order from orderbook
    RemoveOrder(removeorder::RemoveOrder),

    /// List all orders from particular schema compatible sg
    ListOrders(listorders::ListOrder),
}

pub async fn dispatch(orderbook: Orderbook) -> Result<()> {
    match orderbook {
        Orderbook::Deposit(deposit) => {
            let _ = deposit::handle_deposit(deposit).await;
            Ok(())
        }
        Orderbook::Withdraw(withdraw) => {
            let _ = withdraw::handle_withdraw(withdraw).await;
            Ok(())
        }
        Orderbook::AddOrder(order) => {
            let _ = addorder::handle_add_order(order).await;
            Ok(())
        }
        Orderbook::RemoveOrder(order) => {
            let _ = removeorder::handle_remove_order(order).await;
            Ok(())
        }
        Orderbook::ListOrders(listorders) => {
            let _ = listorders::handle_list_order(listorders).await;
            Ok(())
        }
    }
}

pub async fn main() -> Result<()> {
    let cli = Cli::parse();
    dispatch(cli.orderbook).await
}
