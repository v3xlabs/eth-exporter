use alloy::primitives::{Address, U256};

pub enum ERC20 {}

pub struct ExchangeRate {
    pub token_a: Address,
    pub token_b: Address,
    pub rate: U256,
}
