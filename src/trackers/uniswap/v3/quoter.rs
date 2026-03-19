use alloy::{
    primitives::{Address, BlockNumber, U256, U512},
    providers::DynProvider,
};

use crate::{
    token::local::LocalTokenOrFiat,
    trackers::{Quoter, RateDirection, uniswap::v3::UniswapV3Selector},
};
use alloy::sol;

sol! {
    #[sol(rpc)]
    contract UniswapV3Pool {
         function slot0() public view returns (uint160 sqrtPriceX96, int24 tick, uint16 observationIndex, uint16 observationCardinality, uint16 observationCardinalityNext, uint8 feeProtocol, bool unlocked);
         function token0() public view returns (address);
         function token1() public view returns (address);
    }
}

#[derive(Debug, Clone)]
pub struct UniswapV3Quoter {
    pub pool_address: Address,
    pub token0: Address,
    pub token1: Address,
    pub provider: DynProvider,
}

impl UniswapV3Quoter {
    pub async fn from_selector(provider: &DynProvider, selector: UniswapV3Selector) -> Self {
        let pool_address = selector.pool_address;
        let pool = UniswapV3Pool::new(pool_address, provider);
        let token0 = pool.token0().call().await.unwrap();
        let token1 = pool.token1().call().await.unwrap();
        Self {
            pool_address,
            token0,
            token1,
            provider: provider.clone(),
        }
    }
}

impl Quoter for UniswapV3Quoter {
    fn get_slug(&self) -> String {
        format!("uniswap_v3:{}", self.pool_address)
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
        let pool = UniswapV3Pool::new(self.pool_address, &self.provider);
        let slot0 = pool.slot0().block(block.into()).call().await.unwrap();
        let sqrt_price_x96 = U512::from(slot0.sqrtPriceX96);
        let q192 = U512::from(1) << 192;
        let sqrt_price_squared = sqrt_price_x96 * sqrt_price_x96;

        let price0_in_1_raw = (sqrt_price_squared * U512::from(amount_in)) / q192;
        let price1_in_0_raw = (q192 * U512::from(amount_in)) / sqrt_price_squared;

        match direction {
            RateDirection::Forward => U256::from(price0_in_1_raw),
            RateDirection::Reverse => U256::from(price1_in_0_raw),
        }
    }
}
