use actix_web::{post, web, App, HttpResponse, HttpServer, Responder, Error};
use std::convert::TryFrom;
use std::str::FromStr;
use ethers::types::Address;
use ethers::utils::parse_units;
use ethers::{providers::{Provider, Middleware, Http}, types::{H160,U256}};
use ethers_signers::{Ledger, HDPath};
use crate::cli::deposit::Deposit;
use ethers::prelude::SignerMiddleware ; 

use crate::orderbook::deposit::v3::deposit_token;
use crate::tokens::approve_tokens;
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
#[display(fmt = "my error: {}", message)]
struct DepositErr {
    message: &'static str
}

impl actix_web::ResponseError for DepositErr {}

#[post("/api/deposit")]
async fn rpc_deposit(deposit: web::Json<Deposit>) -> Result<HttpResponse, DepositErr> {

    let orderbook_address = match Address::from_str(&deposit.orderbook) {
        Ok(address) => {
            address
        },
        Err(_) => {
            return Err(DepositErr{message: "\n ❌Incorrect orderbook address."});
            // return Err(anyhow!("\n ❌Incorrect orderbook address.")) ;
        }
    };

    let token_address = match Address::from_str(&deposit.token_address) {
        Ok(address) => {
            address
        },
        Err(_) => {
            return Err(DepositErr{message: "\n ❌Incorrect token address."});
            // return Err(anyhow!("\n ❌Incorrect token address.")) ;
        }
    };  

    let token_amount: U256 = match parse_units(deposit.amount.clone(),deposit.token_decimals.clone()) {
        Ok(amount) => amount.into() ,
        Err(_) => {
            return Err(DepositErr{message: "\n ❌Incorrect amount."});
            // return Err(anyhow!("\n ❌Incorrect amount.")) ;
        }
    } ;

    let vault_id = match deposit.vault_id.clone() {
        Some(val) => {
            match U256::from_dec_str(&val) {
                Ok(id) => id ,
                Err(_) => {
                    return Err(DepositErr{message: "\n ❌Invalid vault id."});
                    // return Err(anyhow!("\n ❌Invalid vault id.")) ;
                }
            }
        } ,
        None => {
            U256::from(H160::random().as_bytes()) 
        }
    } ;  

    let rpc_url = deposit.rpc_url.clone().unwrap(); 
    let provider = Provider::<Http>::try_from(rpc_url.clone())
    .expect("\n❌Could not instantiate HTTP Provider");  

    let chain_id = provider.get_chainid().await.unwrap().as_u64() ; 
    let wallet= Ledger::new(HDPath::LedgerLive(0), chain_id.clone()).await.map_err(|_| DepositErr{message: "\n❌Could not instantiate ledger wallet"})?;   
    let wallet_address =  wallet.get_address().await.unwrap() ; 

    let client = SignerMiddleware::new_with_provider_chain(provider, wallet).await.unwrap();     

    // Approve token for deposit 
    let approve_tx = approve_tokens(
        token_address.clone() ,
        token_amount.clone(),
        orderbook_address.clone() ,
        rpc_url.clone(),
        wallet_address,
        deposit.blocknative_api_key.clone()
    ).await.unwrap() ;

    let approve_tx = client.send_transaction(approve_tx, None).await;   
    match approve_tx {
        Ok(approve_tx) => {
            let _ = approve_tx.confirmations(1).await ;
        },
        Err(_) => {
            return Err(DepositErr{message: "\n ❌Token approval failed"});
        }
    }

    // Deposit tokens
    let deposit_tx = deposit_token(
        token_address,
        token_amount,
        vault_id,
        orderbook_address,
        rpc_url,
        deposit.blocknative_api_key.clone()
    ).await.unwrap() ;

    let deposit_tx = client.send_transaction(deposit_tx, None).await;  

    match deposit_tx {
        Ok(deposit_tx) => {
            let _ = deposit_tx.confirmations(1).await ;
        },
        Err(_) => {
            return Err(DepositErr{message: "\n ❌Failed to deposit tokens"});
        }
    }

    Ok(HttpResponse::Ok().finish())
}