use alloy_primitives::{Address, U256};
use anyhow::Result;
use rain_orderbook_bindings::{IOrderBookV3::depositCall, IERC20::approveCall};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Clone, Serialize, Deserialize)]
pub struct DepositArgs {
    pub token: String,
    pub vault_id: u64,
    pub amount: u64,
}

impl TryInto<depositCall> for DepositArgs {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<depositCall> {
        Ok(depositCall {
            token: self.token.parse()?,
            vaultId: U256::from(self.vault_id),
            amount: U256::from(self.amount),
        })
    }
}

impl DepositArgs {
    pub fn try_into_approve_call(self, spender: Address) -> Result<approveCall> {
        Ok(approveCall {
            spender,
            amount: U256::from(self.amount),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{hex, Address};

    #[test]
    fn test_deposit_args_try_into() {
        let args = DepositArgs {
            token: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            vault_id: 42,
            amount: 100,
        };

        let result: Result<depositCall, _> = args.try_into();

        match result {
            Ok(_) => (),
            Err(e) => panic!("Unexpected error: {}", e),
        }

        assert!(result.is_ok());

        let deposit_call = result.unwrap();
        assert_eq!(
            deposit_call.token,
            "0x1234567890abcdef1234567890abcdef12345678"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(deposit_call.vaultId, U256::from(42));
        assert_eq!(deposit_call.amount, U256::from(100));
    }

    #[test]
    fn test_deposit_args_try_into_approve_call() {
        let args = DepositArgs {
            token: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            vault_id: 42,
            amount: 100,
        };
        let spender_address = Address::repeat_byte(0x11);
        let result: Result<approveCall, _> = args.try_into_approve_call(spender_address);

        match result {
            Ok(_) => (),
            Err(e) => panic!("Unexpected error: {}", e),
        }

        assert!(result.is_ok());

        let approve_call = result.unwrap();
        assert_eq!(approve_call.amount, U256::from(100));
        assert_eq!(
            approve_call.spender,
            hex!("1111111111111111111111111111111111111111")
        );
    }
}
