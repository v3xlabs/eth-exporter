use alloy::{primitives::map::HashMap, providers::DynProvider};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::Deserialize;

use crate::trackers::{Quoter, QuoterInstance, erc4626::{ERC4626Config, ERC4626Quoter}, fixed::FixedTracker, uniswap::v2::quoter::{UniswapV2Config, UniswapV2Quoter}};

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
    pub erc4626: Vec<ERC4626Config>,
}

impl TrackersConfig {
    pub async fn all(&self, provider: &DynProvider) -> Vec<QuoterInstance> {
        let mut quoters = Vec::new();
        for tracker in &self.fixed {
            quoters.push(QuoterInstance::Fixed(tracker.clone()));
        }

        for uni_quoters in self.uniswap_v2.pairs.iter() {
            let quoter = UniswapV2Quoter::from_selector(provider, uni_quoters.clone()).await;
            quoters.push(QuoterInstance::UniswapV2(quoter));
        }

        for erc4626_config in &self.erc4626 {
            let quoter = ERC4626Quoter::new(erc4626_config.vault_address, provider).await;
            quoters.push(QuoterInstance::ERC4626(quoter));
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
