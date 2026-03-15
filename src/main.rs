use alloy::primitives::address;
use futures::StreamExt;

pub mod shared;
// #[cfg(test)]
pub mod tests;
pub mod uniswap;

#[tokio::main]
pub async fn main() {
    println!("Hello, world!");

    let factory_address = address!("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f");
    let provider = tests::get_test_provider().await;
    let mut pairs = Box::pin(uniswap::v2::factory::fetch_all_pairs(provider, factory_address));

    while let Some(pair) = pairs.as_mut().next().await {
        println!("pair: {:?}", pair);
    }
}
