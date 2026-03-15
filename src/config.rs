use alloy::{primitives::map::HashMap, providers::DynProvider};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::Deserialize;

use crate::{trackers::{QuoterInstance, fixed::FixedTracker, uniswap::v2::quoter::{UniswapV2Config, UniswapV2Quoter}}, trackers::Quoter};

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    pub chains: HashMap<String, ChainConfig>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub rpc_url: String,
    pub tokens: Vec<TokenConfig>,
    pub trackers: TrackersConfig,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct TrackersConfig {
    pub fixed: Vec<FixedTracker>,
    pub uniswap_v2: UniswapV2Config,
}

impl TrackersConfig {
    pub async fn all(&self, provider: &Box<DynProvider>) -> Vec<QuoterInstance> {
        let mut quoters = Vec::new();
        for tracker in &self.fixed {
            quoters.push(QuoterInstance::Fixed(tracker.clone()));
        }

        for uni_quoters in self.uniswap_v2.pairs.iter() {
            let quoter = UniswapV2Quoter::from_selector(provider.clone(), uni_quoters.clone()).await;
            quoters.push(QuoterInstance::UniswapV2(quoter));
        }

        quoters
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct TokenConfig {
    pub address: String,
    pub slug: Option<String>,
    pub decimals: u8,
}

impl Config {
    pub async fn load(path: &str) -> Self {
        let figment = Figment::new().merge(Toml::file(path));
        figment.extract::<Config>().unwrap()
    }
}
