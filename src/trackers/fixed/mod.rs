use alloy::primitives::{BlockNumber, U256};
use serde::Deserialize;

use crate::{token::local::LocalTokenOrFiat, trackers::{Quoter, RateDirection}};

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
        (self.token_in.clone(), self.token_out.clone())
    }

    async fn get_rate(&self, amount_in: U256, direction: RateDirection, _block: BlockNumber) -> U256 {
        match direction {
            RateDirection::Forward => {
                U256::from(self.fixed_rate * amount_in.to_string().parse::<f64>().unwrap())
            }
            RateDirection::Reverse => {
                U256::from(1.0 / self.fixed_rate * amount_in.to_string().parse::<f64>().unwrap())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy::primitives::address;

    use super::*;

    #[tokio::test]
    async fn test_get_rate() {
        let tracker = FixedTracker {
            token_in: LocalTokenOrFiat::ERC20 { address: address!("0x0000000000000000000000000000000000000001") },
            token_out: LocalTokenOrFiat::ERC20 { address: address!("0x0000000000000000000000000000000000000002") },
            fixed_rate: 2.0,
        };

        let forwards = tracker.get_rate(U256::from(100), RateDirection::Forward, 100).await;
        let backwards = tracker.get_rate(U256::from(100), RateDirection::Reverse, 100).await;

        assert_eq!(forwards, U256::from(200));
        assert_eq!(backwards, U256::from(50));
    }
}
