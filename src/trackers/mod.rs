use alloy::{primitives::{BlockNumber, U256}};

use crate::{token::local::LocalTokenOrFiat, trackers::{erc4626::ERC4626Quoter, fixed::FixedTracker, uniswap::{v2::quoter::UniswapV2Quoter, v3::quoter::UniswapV3Quoter}}};

pub mod fixed;
pub mod uniswap;
pub mod erc4626;

pub enum RateDirection {
    Forward,
    Reverse,
}

pub trait Quoter: Send + Sync {
    fn get_tokens(&self) -> (LocalTokenOrFiat, LocalTokenOrFiat);
    async fn get_rate(&self, amount_in: U256, direction: RateDirection, block: BlockNumber) -> U256;
    fn get_slug(&self) -> String;
}

#[derive(Debug, Clone)]
pub enum QuoterInstance {
    Fixed(FixedTracker),
    UniswapV2(UniswapV2Quoter),
    UniswapV3(UniswapV3Quoter),
    ERC4626(ERC4626Quoter),
}

impl Quoter for QuoterInstance {
    fn get_slug(&self) -> String {
        match self {
            QuoterInstance::Fixed(tracker) => tracker.get_slug(),
            QuoterInstance::UniswapV2(quoter) => quoter.get_slug(),
            QuoterInstance::UniswapV3(quoter) => quoter.get_slug(),
            QuoterInstance::ERC4626(quoter) => quoter.get_slug(),
        }
    }

    fn get_tokens(&self) -> (LocalTokenOrFiat, LocalTokenOrFiat) {
        match self {
            QuoterInstance::Fixed(tracker) => tracker.get_tokens(),
            QuoterInstance::UniswapV2(quoter) => quoter.get_tokens(),
            QuoterInstance::UniswapV3(quoter) => quoter.get_tokens(),
            QuoterInstance::ERC4626(quoter) => quoter.get_tokens(),
        }
    }

    async fn get_rate(&self, amount_in: U256, direction: RateDirection, block: BlockNumber) -> U256 {
        match self {
            QuoterInstance::Fixed(tracker) => tracker.get_rate(amount_in, direction, block).await,
            QuoterInstance::UniswapV2(quoter) => quoter.get_rate(amount_in, direction, block).await,
            QuoterInstance::UniswapV3(quoter) => quoter.get_rate(amount_in, direction, block).await,
            QuoterInstance::ERC4626(quoter) => quoter.get_rate(amount_in, direction, block).await,
        }
    }
}
