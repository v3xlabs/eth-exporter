use std::future::Future;

use alloy::{primitives::{Address, U256, address}, providers::DynProvider};
use serde::Deserialize;
use super::pair::UniswapV2Pair::{self, UniswapV2PairInstance};

use crate::shared::{quoter::Quoter, token::LocalTokenOrFiat};

#[derive(Debug, Deserialize, PartialEq)]
pub struct UniswapV2Config {
    pub factory_address: Address,
    pub pairs: Vec<UniswapV2Selector>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum UniswapV2Selector {
    IO {
        token_in: Address,
        token_out: Address,
    },
    Pair {
        pair_address: Address,
    }
}

#[derive(Debug, Clone)]
pub struct UniswapV2Quoter {
    pub pair_address: Address,
    pub token0: Address,
    pub token1: Address,
}

impl UniswapV2Quoter {
    pub async fn from_contract(contract: UniswapV2PairInstance<Box<DynProvider>>) -> Self {
        let pair_address = *contract.address();
        let token0 = contract.token0().call().await.unwrap();
        let token1 = contract.token1().call().await.unwrap();

        Self { pair_address, token0, token1 }
    }
}

impl Quoter for UniswapV2Quoter {
    fn get_slug(&self) -> String {
        format!("uniswap_v2:{}:{}:{}", self.pair_address, self.token0, self.token1)
    }

    fn get_tokens(&self) -> (LocalTokenOrFiat, LocalTokenOrFiat) {
        (self.token0.into(), self.token1.into())
    }

    async fn get_rate(&self, amount_in: U256) -> U256 {
        // let fr = &pair;

        // let rate = fr.getRate(amount_in).call().await?;
        // Ok(rate)

        U256::from(0)
    }
}

impl UniswapV2Quoter {
    pub async fn from_selector(provider: Box<DynProvider>, selector: UniswapV2Selector) -> Self {
        let factory_address = address!("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f");

        match selector {
            UniswapV2Selector::IO { token_in, token_out } => {
                let pair_address = super::factory::fetch_pair(&provider, factory_address, token_in, token_out).await.unwrap();

                let (token0, token1) = if token_in < token_out { (token_in, token_out) } else { (token_out, token_in) };

                Self { pair_address, token0, token1 }
            }
            UniswapV2Selector::Pair { pair_address } => {
                let pair = UniswapV2Pair::new(pair_address, provider);

                Self::from_contract(pair).await
            }
        }
    }
}