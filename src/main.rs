use alloy::{primitives::{U256, address}, providers::{Provider, ProviderBuilder}};
use futures::StreamExt;

use crate::{config::Config, shared::quoter::Quoter};

pub mod shared;
// #[cfg(test)]
pub mod tests;
pub mod config;
pub mod trackers;

#[tokio::main]
pub async fn main() {
    println!("Hello, world!");

    let config = Config::load("config.toml").await;

    for (chain_slug, chain_config) in config.chains {
        println!("chain: {:?}", chain_slug);
        let url = chain_config.rpc_url;
        let provider = ProviderBuilder::new().connect(&url).await.unwrap();

        println!("tokens: {:?}", chain_config.tokens.len());
        for token_config in chain_config.tokens {
            let token_address = token_config.address;
            println!("token: {:?}", token_address);
        }

        let box_provider = Box::new(provider.erased());
        // TODO: turn all trackers into quoters
        for tracker in chain_config.trackers.all(&box_provider).await {
            println!("tracker: {:?}", tracker.get_slug());
            let tokens = tracker.get_tokens();
            println!("tokens: {:?}", tokens);
            let amount = U256::from(10).pow(U256::from(6));
            let rate = tracker.get_rate(amount).await;
            println!("rate: {:?}", rate);
            //
        }
    }
}
