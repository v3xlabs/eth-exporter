use super::pair::UniswapV2Pair::{self, UniswapV2PairInstance};
use alloy::{
    primitives::{address, Address, BlockNumber, U256, U512},
    providers::DynProvider,
};
use serde::Deserialize;

use crate::{
    token::local::LocalTokenOrFiat,
    trackers::{Quoter, RateDirection},
};

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
    },
}

#[derive(Debug, Clone)]
pub struct UniswapV2Quoter {
    pub pair_address: Address,
    pub token0: Address,
    pub token1: Address,
    pub provider: DynProvider,
}

impl UniswapV2Quoter {
    pub async fn from_contract(
        contract: UniswapV2PairInstance<&DynProvider>,
        provider: &DynProvider,
    ) -> Self {
        let pair_address = *contract.address();
        let token0 = contract.token0().call().await.unwrap();
        let token1 = contract.token1().call().await.unwrap();

        Self {
            pair_address,
            token0,
            token1,
            provider: provider.clone(),
        }
    }
}

impl UniswapV2Quoter {
    pub async fn from_selector(provider: &DynProvider, selector: UniswapV2Selector) -> Self {
        let factory_address = address!("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f");

        match selector {
            UniswapV2Selector::IO {
                token_in,
                token_out,
            } => {
                let pair_address =
                    super::factory::fetch_pair(provider, factory_address, token_in, token_out)
                        .await
                        .unwrap();

                let (token0, token1) = if token_in < token_out {
                    (token_in, token_out)
                } else {
                    (token_out, token_in)
                };

                Self {
                    pair_address,
                    token0,
                    token1,
                    provider: provider.clone(),
                }
            }
            UniswapV2Selector::Pair { pair_address } => {
                let pair = UniswapV2Pair::new(pair_address, provider);

                Self::from_contract(pair, provider).await
            }
        }
    }
}

impl Quoter for UniswapV2Quoter {
    fn get_slug(&self) -> String {
        format!(
            "uniswap_v2:{}:{}:{}",
            self.pair_address, self.token0, self.token1
        )
    }

    fn get_tokens(&self) -> (LocalTokenOrFiat, LocalTokenOrFiat) {
        (self.token0.into(), self.token1.into())
    }

    async fn get_rate(
        &self,
        amount_in: U256,
        direction: RateDirection,
        block: BlockNumber,
    ) -> U256 {
        let pair = UniswapV2Pair::new(self.pair_address, &self.provider);
        let reserves = pair.getReserves().call().block(block.into()).await.unwrap();
        let reserve0 = U512::from(reserves.reserve0);
        let reserve1 = U512::from(reserves.reserve1);
        let amount_in = U512::from(amount_in);
        let scale = U512::from(10).pow(U512::from(8));
        match direction {
            RateDirection::Forward => {
                // let rate = reserve0 * scale / reserve1;

                // let x = U512::from(amount_in) * rate / scale;

                // x.to_string().parse::<U256>().unwrap()

                println!("amount_in: {:?}", amount_in);
                println!("reserve0: {:?}", reserve0);
                println!("reserve1: {:?}", reserve1);
                println!("scale: {:?}", scale);

                let numerator = amount_in * reserve1;
                let denominator = reserve0;

                let amount_out = numerator / denominator;

                U256::from(amount_out)
            }
            RateDirection::Reverse => {
                let numerator = amount_in * reserve0;
                let denominator = reserve1;

                let amount_out = numerator / denominator;

                U256::from(amount_out)
            }
        }
    }
}
