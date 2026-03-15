use alloy::primitives::map::HashMap;
use figment::{Figment, providers::{Format, Toml}};
use serde::Deserialize;

use crate::{quoters::{fixed::FixedTrackerConfig, uniswap::v2::quoter::{UniswapV2Config, UniswapV2Quoter, UniswapV2Selector}}, shared::quoter::Quoter};

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    pub chains: HashMap<String, ChainConfig>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub rpc_url: String,
    pub tokens: HashMap<String, TokenConfig>,
    pub trackers: TrackersConfig,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct TrackersConfig {
    pub fixed: Vec<FixedTrackerConfig>,
    pub uniswap_v2: UniswapV2Config,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct TokenConfig {
    pub address: String,
}

impl Config {
    pub async fn load(path: &str) -> Self {
        let figment = Figment::new().merge(Toml::file(path));
        let config = figment.extract::<Config>().unwrap();
        config
    }
}
