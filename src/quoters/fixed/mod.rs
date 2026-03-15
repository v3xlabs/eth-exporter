use alloy::{primitives::{Address, U256}, providers::DynProvider};
use serde::Deserialize;

use crate::shared::quoter::Quoter;

#[derive(Debug, PartialEq)]
pub struct FixedTracker {
    pub token_in: String,
    pub token_out: String,
    pub fixed_rate: f64,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct FixedTrackerConfig {
    pub token_in: String,
    pub token_out: String,
    pub fixed_rate: f64,
}

impl Quoter for FixedTracker {
    type Selector = FixedTrackerConfig;

    fn get_slug(&self) -> String {
        format!("fixed:{}:{}", self.token_in, self.token_out)
    }

    async fn from_selector(provider: Box<DynProvider>, selector: Self::Selector) -> Self {
        Self {
            token_in: selector.token_in,
            token_out: selector.token_out,
            fixed_rate: selector.fixed_rate,
        }
    }

    fn get_tokens(&self) -> (Address, Address) {
        (self.token_in.parse().unwrap(), self.token_out.parse().unwrap())
    }

    async fn get_rate(&self, amount_in: U256) -> U256 {
        U256::from(self.fixed_rate * amount_in.to_string().parse::<f64>().unwrap())
    }
}
