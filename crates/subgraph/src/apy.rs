use crate::{
    types::common::{Erc20, Order, Trade},
    utils::{one_18, to_18_decimals, year_18},
    vol::{get_vaults_vol, VaultVolume},
    OrderbookSubgraphClientError,
};
use alloy::primitives::{
    utils::{parse_units, ParseUnits},
    I256, U256,
};
use chrono::TimeDelta;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
};
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct TokenVaultAPY {
    pub id: String,
    pub token: Erc20,
    #[typeshare(typescript(type = "number"))]
    pub start_time: u64,
    #[typeshare(typescript(type = "number"))]
    pub end_time: u64,
    #[typeshare(typescript(type = "string"))]
    pub net_vol: I256,
    #[typeshare(typescript(type = "string"))]
    pub capital: I256,
    #[typeshare(typescript(type = "string"))]
    pub apy: Option<I256>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct DenominatedAPY {
    #[typeshare(typescript(type = "string"))]
    pub apy: I256,
    pub token: Erc20,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct OrderAPY {
    pub order_id: String,
    pub order_hash: String,
    pub denominated_apy: Option<DenominatedAPY>,
    #[typeshare(typescript(type = "number"))]
    pub start_time: u64,
    #[typeshare(typescript(type = "number"))]
    pub end_time: u64,
    pub inputs_token_vault_apy: Vec<TokenVaultAPY>,
    pub outputs_token_vault_apy: Vec<TokenVaultAPY>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TokenPair {
    input: Erc20,
    output: Erc20,
}

/// Given an order and its trades and optionally a timeframe, will calculates
/// the APY for each of the entire order and for each of its vaults
/// Trades must be sorted indesc order by timestamp, this is the case if
/// queried from subgraph using this lib functionalities
pub fn get_order_apy(
    order: &Order,
    trades: &[Trade],
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<OrderAPY, OrderbookSubgraphClientError> {
    if trades.is_empty() {
        return Ok(OrderAPY {
            order_id: order.id.0.clone(),
            order_hash: order.order_hash.0.clone(),
            start_time: start_timestamp.unwrap_or(0),
            end_time: end_timestamp.unwrap_or(chrono::Utc::now().timestamp() as u64),
            inputs_token_vault_apy: vec![],
            outputs_token_vault_apy: vec![],
            denominated_apy: None,
        });
    }
    let vols = get_vaults_vol(trades)?;
    let token_vaults_apy = get_token_vaults_apy(trades, &vols, start_timestamp, end_timestamp)?;

    // build an OrderApy struct
    let mut start_time = u64::MAX;
    let mut end_time = 0_u64;
    let mut inputs: Vec<TokenVaultAPY> = vec![];
    let mut outputs: Vec<TokenVaultAPY> = vec![];
    for item in &token_vaults_apy {
        if item.start_time < start_time {
            start_time = item.start_time;
        }
        if item.end_time > end_time {
            end_time = item.end_time;
        }
        if order
            .inputs
            .iter()
            .any(|v| v.vault_id.0 == item.id && v.token == item.token)
        {
            inputs.push(item.clone());
        }
        if order
            .outputs
            .iter()
            .any(|v| v.vault_id.0 == item.id && v.token == item.token)
        {
            outputs.push(item.clone());
        }
    }
    let mut order_apy = OrderAPY {
        order_id: order.id.0.clone(),
        order_hash: order.order_hash.0.clone(),
        start_time,
        end_time,
        inputs_token_vault_apy: inputs,
        outputs_token_vault_apy: outputs,
        denominated_apy: None,
    };

    // get pairs ratios
    let pair_ratio_map = get_pairs_ratio(&order_apy, trades);

    // try to calculate all vaults capital and volume denominated into each of
    // the order's tokens by checking if there is direct ratio between the tokens,
    // multi path ratios are ignored currently and results in None for the APY.
    // if there is a success for any of the denomination tokens, gather it in order
    // of its net vol and pick the one with highest net vol.
    // if there was no success with any of the order's tokens, simply return None
    // for the APY.
    let mut ordered_token_net_vol_map = BTreeMap::new();
    let mut full_apy_in_distinct_token_denominations = vec![];
    for token in &token_vaults_apy {
        let mut noway = false;
        let mut combined_capital = I256::ZERO;
        let mut combined_annual_rate_vol = I256::ZERO;
        let mut token_net_vol_map_converted_in_current_denomination = BTreeMap::new();
        for token_vault in &token_vaults_apy {
            // time to year ratio
            let annual_rate = annual_rate(token_vault.start_time, token_vault.end_time);

            // sum up all token vaults' capitals and vols in the current's iteration
            // token denomination by using the direct ratio between the tokens
            if token_vault.token == token.token {
                combined_capital += token_vault.capital;
                combined_annual_rate_vol += token_vault
                    .net_vol
                    .saturating_mul(one_18().get_signed())
                    .saturating_div(annual_rate);
                token_net_vol_map_converted_in_current_denomination
                    .insert(token_vault.net_vol, &token.token);
            } else {
                let pair = TokenPair {
                    input: token.token.clone(),
                    output: token_vault.token.clone(),
                };
                // convert to current denomination by the direct pair ratio if exists
                if let Some(Some(ratio)) = pair_ratio_map.get(&pair) {
                    combined_capital += token_vault
                        .capital
                        .saturating_mul(*ratio)
                        .saturating_div(one_18().get_signed());
                    combined_annual_rate_vol += token_vault
                        .net_vol
                        .saturating_mul(*ratio)
                        .saturating_div(one_18().get_signed())
                        .saturating_mul(one_18().get_signed())
                        .saturating_div(annual_rate);
                    token_net_vol_map_converted_in_current_denomination.insert(
                        token_vault
                            .net_vol
                            .saturating_mul(*ratio)
                            .saturating_div(one_18().get_signed()),
                        &token_vault.token,
                    );
                } else {
                    noway = true;
                    break;
                }
            }
        }

        // for every success apy calc in a token denomination, gather them in BTreeMap
        // this means at the end we have all the successful apy calculated in each of
        // the order's io tokens in order from highest to lowest.
        if !noway {
            if let Some(apy) = combined_annual_rate_vol
                .saturating_mul(one_18().get_signed())
                .checked_div(combined_capital)
            {
                full_apy_in_distinct_token_denominations.push(DenominatedAPY {
                    apy,
                    token: token.token.clone(),
                });
            }
        } else {
            token_net_vol_map_converted_in_current_denomination.clear();
        }

        // if we already have ordered token net vol in a denomination
        // we dont need them in other denominations in order to pick
        // the highest vol token as settelement denomination
        if ordered_token_net_vol_map.is_empty() {
            ordered_token_net_vol_map.extend(token_net_vol_map_converted_in_current_denomination);
        }
    }

    // pick the denomination with highest net vol by iterating over tokens with
    // highest vol to lowest and pick the first matching matching one
    for (_, &token) in ordered_token_net_vol_map.iter().rev() {
        if let Some(denominated_apy) = full_apy_in_distinct_token_denominations
            .iter()
            .find(|&v| &v.token == token)
        {
            order_apy.denominated_apy = Some(denominated_apy.clone());
            // return early as soon as a match is found
            return Ok(order_apy);
        }
    }

    Ok(order_apy)
}

/// Calculates each token vault apy at the given timeframe
/// Trades must be sorted indesc order by timestamp, this is
/// the case if queried from subgraph using this lib functionalities
pub fn get_token_vaults_apy(
    trades: &[Trade],
    vols: &[VaultVolume],
    start_timestamp: Option<u64>,
    end_timestamp: Option<u64>,
) -> Result<Vec<TokenVaultAPY>, OrderbookSubgraphClientError> {
    let mut token_vaults_apy: Vec<TokenVaultAPY> = vec![];
    for vol in vols {
        // this token vault trades in desc order by timestamp
        let vault_trades = trades
            .iter()
            .filter(|v| {
                (v.input_vault_balance_change.vault.vault_id.0 == vol.id
                    && v.input_vault_balance_change.vault.token == vol.token)
                    || (v.output_vault_balance_change.vault.vault_id.0 == vol.id
                        && v.output_vault_balance_change.vault.token == vol.token)
            })
            .collect::<Vec<&Trade>>();

        // this token vault first trade, indictaes the start time
        // to find the end of the first day to find the starting capital
        let first_trade = vault_trades[vault_trades.len() - 1];
        let first_day_last_trade = vault_trades
            .iter()
            .filter(|v| {
                u64::from_str(&v.timestamp.0).unwrap()
                    <= u64::from_str(&first_trade.timestamp.0).unwrap()
                        + TimeDelta::days(1).num_seconds() as u64
            })
            .collect::<Vec<&&Trade>>()[0];

        // vaults starting capital at end of first day of its first ever trade
        // as 18 point decimals
        let vault_balance_change = if first_day_last_trade
            .input_vault_balance_change
            .vault
            .vault_id
            .0
            == vol.id
            && first_day_last_trade.input_vault_balance_change.vault.token == vol.token
        {
            &first_day_last_trade.input_vault_balance_change
        } else {
            &first_day_last_trade.output_vault_balance_change
        };
        let starting_capital = U256::from_str(&vault_balance_change.new_vault_balance.0)
            .ok()
            .and_then(|amount| {
                to_18_decimals(
                    ParseUnits::U256(amount),
                    vault_balance_change
                        .vault
                        .token
                        .decimals
                        .as_ref()
                        .map(|v| v.0.as_str())
                        .unwrap_or("18"),
                )
                .ok()
            });

        // convert net vol to 18 decimals point
        let net_vol = to_18_decimals(
            ParseUnits::I256(vol.net_vol),
            vol.token
                .decimals
                .as_ref()
                .map(|v| v.0.as_str())
                .unwrap_or("18"),
        )
        .ok();

        // the time range for this token vault
        let mut start = u64::from_str(&first_trade.timestamp.0)?;
        start_timestamp.inspect(|t| {
            if start > *t {
                start = *t;
            }
        });
        let end = end_timestamp.unwrap_or(chrono::Utc::now().timestamp() as u64);

        // this token vault apy in 18 decimals point
        let apy = starting_capital
            .zip(net_vol)
            .and_then(|(starting_capital, net_vol)| {
                (!starting_capital.is_zero())
                    .then_some(
                        net_vol
                            .get_signed()
                            .saturating_mul(one_18().get_signed())
                            .saturating_div(starting_capital.get_signed())
                            .saturating_mul(one_18().get_signed())
                            .checked_div(annual_rate(start, end)),
                    )
                    .flatten()
            });

        // this token vault apy
        token_vaults_apy.push(TokenVaultAPY {
            id: vol.id.clone(),
            token: vol.token.clone(),
            start_time: start,
            end_time: end,
            apy,
            net_vol: net_vol.unwrap_or(ParseUnits::I256(I256::ZERO)).get_signed(),
            capital: starting_capital
                .unwrap_or(ParseUnits::I256(I256::ZERO))
                .get_signed(),
        });
    }

    Ok(token_vaults_apy)
}

/// Calculates an order's pairs' ratios from their last trades in a given list of trades
/// Trades must be sorted indesc order by timestamp, this is the case if queried from subgraph
/// using this lib functionalities
fn get_pairs_ratio(order_apy: &OrderAPY, trades: &[Trade]) -> HashMap<TokenPair, Option<I256>> {
    let mut pair_ratio_map: HashMap<TokenPair, Option<I256>> = HashMap::new();
    for input in &order_apy.inputs_token_vault_apy {
        for output in &order_apy.outputs_token_vault_apy {
            let pair_as_key = TokenPair {
                input: input.token.clone(),
                output: output.token.clone(),
            };
            let inverse_pair_as_key = TokenPair {
                input: output.token.clone(),
                output: input.token.clone(),
            };
            // if not same io token and ratio map doesnt already include them
            if input.token != output.token
                && !(pair_ratio_map.contains_key(&pair_as_key)
                    || pair_ratio_map.contains_key(&inverse_pair_as_key))
            {
                // find this pairs(io or oi) latest tradetrades from list of order's
                // trades, the calculate the pair ratio (in amount/out amount) and
                // its inverse from the latest trade that involes these 2 tokens.
                // this assumes the trades are already in desc order by timestamp which
                // is the case when used this lib query to get them
                let ratio = trades
                    .iter()
                    .find(|v| {
                        (v.input_vault_balance_change.vault.token == input.token
                            && v.output_vault_balance_change.vault.token == output.token)
                            || (v.output_vault_balance_change.vault.token == input.token
                                && v.input_vault_balance_change.vault.token == output.token)
                    })
                    .and_then(|latest_trade| {
                        // convert input and output amounts to 18 decimals point
                        // and then calculate the pair ratio
                        latest_trade.ratio().zip(latest_trade.inverse_ratio()).map(
                            |(ratio, inverse_ratio)| {
                                [I256::from_raw(ratio), I256::from_raw(inverse_ratio)]
                            },
                        )
                    });

                // io
                pair_ratio_map.insert(pair_as_key, ratio.map(|v| v[0]));
                // oi
                pair_ratio_map.insert(inverse_pair_as_key, ratio.map(|v| v[1]));
            }
        }
    }

    pair_ratio_map
}

/// Returns annual rate as 18 point decimals as I256
fn annual_rate(start: u64, end: u64) -> I256 {
    parse_units(&(end - start).to_string(), 18)
        .unwrap()
        .get_signed()
        .saturating_mul(one_18().get_signed())
        .saturating_div(year_18().get_signed())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::common::{
        BigInt, Bytes, Orderbook, TradeEvent, TradeStructPartialOrder, TradeVaultBalanceChange,
        Transaction, Vault, VaultBalanceChangeVault,
    };
    use alloy::primitives::{Address, B256};

    #[test]
    fn test_get_pairs_ratio() {
        let trades = get_trades();
        let [token1, token2] = get_tokens();
        let [vault1, vault2] = get_vault_ids();
        let token_vault1 = TokenVaultAPY {
            id: vault1.to_string(),
            token: token1.clone(),
            start_time: 0,
            end_time: 0,
            net_vol: I256::ZERO,
            capital: I256::ZERO,
            apy: Some(I256::ZERO),
        };
        let token_vault2 = TokenVaultAPY {
            id: vault2.to_string(),
            token: token2.clone(),
            start_time: 0,
            end_time: 0,
            net_vol: I256::ZERO,
            capital: I256::ZERO,
            apy: Some(I256::ZERO),
        };
        let order_apy = OrderAPY {
            order_id: "".to_string(),
            order_hash: "".to_string(),
            denominated_apy: None,
            start_time: 0,
            end_time: 0,
            inputs_token_vault_apy: vec![token_vault1.clone(), token_vault2.clone()],
            outputs_token_vault_apy: vec![token_vault1, token_vault2],
        };
        let result = get_pairs_ratio(&order_apy, &trades);
        let mut expected = HashMap::new();
        expected.insert(
            TokenPair {
                input: token2.clone(),
                output: token1.clone(),
            },
            Some(I256::from_str("285714285714285714").unwrap()),
        );
        expected.insert(
            TokenPair {
                input: token1.clone(),
                output: token2.clone(),
            },
            Some(I256::from_str("3500000000000000000").unwrap()),
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_token_vaults_apy() {
        let trades = get_trades();
        let [token1, token2] = get_tokens();
        let [vault1, vault2] = get_vault_ids();
        let vault_vol1 = VaultVolume {
            id: vault1.to_string(),
            token: token1.clone(),
            total_in: U256::ZERO,
            total_out: U256::ZERO,
            total_vol: U256::ZERO,
            net_vol: I256::from_str("1000000000000000000").unwrap(),
        };
        let vault_vol2 = VaultVolume {
            id: vault2.to_string(),
            token: token2.clone(),
            total_in: U256::ZERO,
            total_out: U256::ZERO,
            total_vol: U256::ZERO,
            net_vol: I256::from_str("2000000000000000000").unwrap(),
        };
        let result =
            get_token_vaults_apy(&trades, &[vault_vol1, vault_vol2], Some(1), Some(10000001))
                .unwrap();
        let expected = vec![
            TokenVaultAPY {
                id: vault1.to_string(),
                token: token1.clone(),
                start_time: 1,
                end_time: 10000001,
                net_vol: I256::from_str("1000000000000000000").unwrap(),
                capital: I256::from_str("5000000000000000000").unwrap(),
                // (1/5) / (10000001_end - 1_start / 31_536_00_year)
                apy: Some(I256::from_str("630720000000000000").unwrap()),
            },
            TokenVaultAPY {
                id: vault2.to_string(),
                token: token2.clone(),
                start_time: 1,
                end_time: 10000001,
                net_vol: I256::from_str("2000000000000000000").unwrap(),
                capital: I256::from_str("5000000000000000000").unwrap(),
                // (2/5) / ((10000001_end - 1_start) / 31_536_00_year)
                apy: Some(I256::from_str("1261440000000000000").unwrap()),
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_order_apy() {
        let order = get_order();
        let trades = get_trades();
        let [token1, token2] = get_tokens();
        let [vault1, vault2] = get_vault_ids();
        let token1_apy = TokenVaultAPY {
            id: vault1.to_string(),
            token: token1.clone(),
            start_time: 1,
            end_time: 10000001,
            net_vol: I256::from_str("5000000000000000000").unwrap(),
            capital: I256::from_str("5000000000000000000").unwrap(),
            apy: Some(I256::from_str("3153600000000000000").unwrap()),
        };
        let token2_apy = TokenVaultAPY {
            id: vault2.to_string(),
            token: token2.clone(),
            start_time: 1,
            end_time: 10000001,
            net_vol: I256::from_str("3000000000000000000").unwrap(),
            capital: I256::from_str("5000000000000000000").unwrap(),
            apy: Some(I256::from_str("1892160000000000000").unwrap()),
        };
        let result = get_order_apy(&order, &trades, Some(1), Some(10000001)).unwrap();
        let expected = OrderAPY {
            order_id: "order-id".to_string(),
            order_hash: "".to_string(),
            start_time: 1,
            end_time: 10000001,
            inputs_token_vault_apy: vec![token1_apy.clone(), token2_apy.clone()],
            outputs_token_vault_apy: vec![token1_apy.clone(), token2_apy.clone()],
            denominated_apy: Some(DenominatedAPY {
                apy: I256::from_str("2172479999999999999").unwrap(),
                token: token2,
            }),
        };

        assert_eq!(result, expected);
    }

    fn get_vault_ids() -> [B256; 2] {
        [
            B256::from_slice(&[0x11u8; 32]),
            B256::from_slice(&[0x22u8; 32]),
        ]
    }
    fn get_tokens() -> [Erc20; 2] {
        let token1_address = Address::from_slice(&[0x11u8; 20]);
        let token2_address = Address::from_slice(&[0x22u8; 20]);
        let token1 = Erc20 {
            id: Bytes(token1_address.to_string()),
            address: Bytes(token1_address.to_string()),
            name: Some("Token1".to_string()),
            symbol: Some("Token1".to_string()),
            decimals: Some(BigInt(18.to_string())),
        };
        let token2 = Erc20 {
            id: Bytes(token2_address.to_string()),
            address: Bytes(token2_address.to_string()),
            name: Some("Token2".to_string()),
            symbol: Some("Token2".to_string()),
            decimals: Some(BigInt(18.to_string())),
        };
        [token1, token2]
    }
    fn get_order() -> Order {
        let [vault_id1, vault_id2] = get_vault_ids();
        let [token1, token2] = get_tokens();
        let vault1 = Vault {
            id: Bytes("".to_string()),
            owner: Bytes("".to_string()),
            vault_id: BigInt(vault_id1.to_string()),
            balance: BigInt("".to_string()),
            token: token1,
            orderbook: Orderbook {
                id: Bytes("".to_string()),
            },
            orders_as_output: vec![],
            orders_as_input: vec![],
            balance_changes: vec![],
        };
        let vault2 = Vault {
            id: Bytes("".to_string()),
            owner: Bytes("".to_string()),
            vault_id: BigInt(vault_id2.to_string()),
            balance: BigInt("".to_string()),
            token: token2,
            orderbook: Orderbook {
                id: Bytes("".to_string()),
            },
            orders_as_output: vec![],
            orders_as_input: vec![],
            balance_changes: vec![],
        };
        Order {
            id: Bytes("order-id".to_string()),
            order_bytes: Bytes("".to_string()),
            order_hash: Bytes("".to_string()),
            owner: Bytes("".to_string()),
            outputs: vec![vault1.clone(), vault2.clone()],
            inputs: vec![vault1, vault2],
            orderbook: Orderbook {
                id: Bytes("".to_string()),
            },
            active: true,
            timestamp_added: BigInt("".to_string()),
            meta: None,
            add_events: vec![],
            trades: vec![],
        }
    }

    fn get_trades() -> Vec<Trade> {
        let bytes = Bytes("".to_string());
        let bigint = BigInt("".to_string());
        let [vault_id1, vault_id2] = get_vault_ids();
        let [token1, token2] = get_tokens();
        let trade1 = Trade {
            id: bytes.clone(),
            order: TradeStructPartialOrder {
                id: bytes.clone(),
                order_hash: bytes.clone(),
            },
            trade_event: TradeEvent {
                sender: bytes.clone(),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
            },
            timestamp: BigInt("1".to_string()),
            orderbook: Orderbook { id: bytes.clone() },
            output_vault_balance_change: TradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: BigInt("-2000000000000000000".to_string()),
                new_vault_balance: BigInt("2000000000000000000".to_string()),
                old_vault_balance: bigint.clone(),
                vault: VaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token1.clone(),
                    vault_id: BigInt(vault_id1.to_string()),
                },
                timestamp: BigInt("1".to_string()),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: BigInt("1".to_string()),
                },
                orderbook: Orderbook { id: bytes.clone() },
            },
            input_vault_balance_change: TradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: BigInt("5000000000000000000".to_string()),
                new_vault_balance: BigInt("2000000000000000000".to_string()),
                old_vault_balance: bigint.clone(),
                vault: VaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token2.clone(),
                    vault_id: BigInt(vault_id2.to_string()),
                },
                timestamp: BigInt("1".to_string()),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: BigInt("1".to_string()),
                },
                orderbook: Orderbook { id: bytes.clone() },
            },
        };
        let trade2 = Trade {
            id: bytes.clone(),
            order: TradeStructPartialOrder {
                id: bytes.clone(),
                order_hash: bytes.clone(),
            },
            trade_event: TradeEvent {
                sender: bytes.clone(),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
            },
            timestamp: BigInt("2".to_string()),
            orderbook: Orderbook { id: bytes.clone() },
            output_vault_balance_change: TradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: BigInt("-2000000000000000000".to_string()),
                new_vault_balance: BigInt("5000000000000000000".to_string()),
                old_vault_balance: bigint.clone(),
                vault: VaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token2.clone(),
                    vault_id: BigInt(vault_id2.to_string()),
                },
                timestamp: BigInt("2".to_string()),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: BigInt("1".to_string()),
                },
                orderbook: Orderbook { id: bytes.clone() },
            },
            input_vault_balance_change: TradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: BigInt("7000000000000000000".to_string()),
                new_vault_balance: BigInt("5000000000000000000".to_string()),
                old_vault_balance: bigint.clone(),
                vault: VaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token1.clone(),
                    vault_id: BigInt(vault_id1.to_string()),
                },
                timestamp: BigInt("2".to_string()),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: BigInt("1".to_string()),
                },
                orderbook: Orderbook { id: bytes.clone() },
            },
        };
        vec![trade2, trade1]
    }
}
