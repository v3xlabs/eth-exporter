use alloy::primitives::U256;

use crate::{token::local::LocalTokenOrFiat, trackers::{fixed::FixedTracker, uniswap::v2::quoter::UniswapV2Quoter}};

pub mod fixed;
pub mod uniswap;

pub trait Quoter: Send + Sync {
    fn get_tokens(&self) -> (LocalTokenOrFiat, LocalTokenOrFiat);
    async fn get_rate(&self, amount_in: U256) -> U256;
    fn get_slug(&self) -> String;
}

#[derive(Debug, Clone)]
pub enum QuoterInstance {
    Fixed(FixedTracker),
    UniswapV2(UniswapV2Quoter),
}

impl Quoter for QuoterInstance {
    fn get_slug(&self) -> String {
        match self {
            QuoterInstance::Fixed(tracker) => tracker.get_slug(),
            QuoterInstance::UniswapV2(quoter) => quoter.get_slug(),
        }
    }

    fn get_tokens(&self) -> (LocalTokenOrFiat, LocalTokenOrFiat) {
        match self {
            QuoterInstance::Fixed(tracker) => tracker.get_tokens(),
            QuoterInstance::UniswapV2(quoter) => quoter.get_tokens(),
        }
    }

    async fn get_rate(&self, amount_in: U256) -> U256 {
        match self {
            QuoterInstance::Fixed(tracker) => tracker.get_rate(amount_in).await,
            QuoterInstance::UniswapV2(quoter) => quoter.get_rate(amount_in).await,
        }
    }
}
