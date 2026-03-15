use alloy::primitives::U256;
use serde::Deserialize;

use crate::{token::local::LocalTokenOrFiat, trackers::Quoter};

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct FixedTracker {
    pub token_in: LocalTokenOrFiat,
    pub token_out: LocalTokenOrFiat,
    pub fixed_rate: f64,
}

impl Quoter for FixedTracker {
    fn get_slug(&self) -> String {
        format!("fixed:{}:{}", self.token_in, self.token_out)
    }

    fn get_tokens(&self) -> (LocalTokenOrFiat, LocalTokenOrFiat) {
        (self.token_in.clone(), self.token_out.clone().into())
    }

    async fn get_rate(&self, amount_in: U256) -> U256 {
        U256::from(self.fixed_rate * amount_in.to_string().parse::<f64>().unwrap())
    }
}
