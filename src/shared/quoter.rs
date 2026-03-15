use alloy::primitives::U256;

use crate::shared::token::LocalTokenOrFiat;

pub trait Quoter: Send + Sync {
    fn get_tokens(&self) -> (LocalTokenOrFiat, LocalTokenOrFiat);
    async fn get_rate(&self, amount_in: U256) -> U256;
    fn get_slug(&self) -> String;
}
