use std::future::Future;

use alloy::{primitives::{Address, U256}, providers::DynProvider};

pub trait Quoter {
    type Selector;

    fn from_selector(provider: Box<DynProvider>, selector: Self::Selector) -> impl Future<Output = Self> + Send;

    fn get_tokens(&self) -> (Address, Address);
    fn get_rate(&self, amount_in: U256) -> impl Future<Output = U256> + Send;
    fn get_slug(&self) -> String;
}
