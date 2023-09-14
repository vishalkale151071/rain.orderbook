use ethers::{providers::{Provider, Middleware, Http}, types::{H160,U256, Eip1559TransactionRequest, Bytes, U64}, utils::parse_units} ; 
use std::{convert::TryFrom, sync::Arc};
use anyhow::anyhow;
use tracing::error;
use crate::{cli::registry::IOrderBookV3, gasoracle::{is_block_native_supported, gas_price_oracle}}; 

pub async fn withdraw_tokens(
    withdraw_token_address : H160 ,
    withdraw_token_amount : U256 ,
    wihtdraw_vault_id : U256,
    orderbook_address : H160,
    rpc_url : String,
    signer_address: H160,
    blocknative_api_key : Option<String>
) -> anyhow::Result<Eip1559TransactionRequest>{ 

    let provider = match Provider::<Http>::try_from(rpc_url.clone()){
        Ok(provider) => {
            provider
        },
        Err(err) => {
            error!("INVALID RPC URL: {}",err) ; 
            return Err(anyhow!(err)) ;
        }
    } ;

    let chain_id = provider.clone().get_chainid().await.unwrap().as_u64();

    let orderbook = IOrderBookV3::new(orderbook_address, Arc::new(provider.clone()));  

    let vault_balance: U256 = orderbook.vault_balance(signer_address, withdraw_token_address, wihtdraw_vault_id).call().await.unwrap() ; 

    if withdraw_token_amount.gt(&vault_balance) {
        error!("INSUFFICIENT VAULT BALANCE FOR WITHDRAWAL"); 
        return Err(anyhow!("INSUFFICIENT VAULT BALANCE FOR WITHDRAWAL")); 
    }

    let withdraw_tx = orderbook.withdraw(withdraw_token_address,wihtdraw_vault_id,withdraw_token_amount) ; 
    let withdraw_data: Bytes = withdraw_tx.calldata().unwrap() ;

    let mut withdraw_tx = Eip1559TransactionRequest::new();
    withdraw_tx.to = Some(orderbook_address.into());
    withdraw_tx.value = Some(U256::zero());
    withdraw_tx.data = Some(withdraw_data);
    withdraw_tx.chain_id = Some(U64::from_dec_str(&chain_id.to_string()).unwrap()); 

    if is_block_native_supported(chain_id) {
        let (max_priority,max_fee) = gas_price_oracle(blocknative_api_key, chain_id).await.unwrap() ; 
        let max_priority: U256 = parse_units(max_priority.to_string(), 9).unwrap().into() ;
        let max_fee: U256 = parse_units(max_fee.to_string(), 9).unwrap().into() ;

        withdraw_tx.max_priority_fee_per_gas = Some(max_priority);
        withdraw_tx.max_fee_per_gas = Some(max_fee);
    }   
    Ok(withdraw_tx)
}