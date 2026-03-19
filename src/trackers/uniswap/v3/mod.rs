use alloy::primitives::Address;
use serde::Deserialize;

pub mod pool;
pub mod factory;
pub mod quoter;

#[derive(Debug, Deserialize, PartialEq)]
pub struct UniswapV3Config {
    pub pools: Vec<UniswapV3Selector>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct UniswapV3Selector {
    pool_address: Address,
}
