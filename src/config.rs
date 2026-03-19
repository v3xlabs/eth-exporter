use alloy::{primitives::map::HashMap, providers::DynProvider};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::Deserialize;

use crate::trackers::{
    erc4626::{ERC4626Config, ERC4626Quoter},
    fixed::FixedTracker,
    uniswap::{
        v2::quoter::{UniswapV2Config, UniswapV2Quoter},
        v3::{quoter::UniswapV3Quoter, UniswapV3Config},
    },
    Quoter, QuoterInstance,
};

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
    pub uniswap_v2: Option<UniswapV2Config>,
    pub uniswap_v3: Option<UniswapV3Config>,
    pub erc4626: Vec<ERC4626Config>,
}

impl TrackersConfig {
    pub async fn all(&self, provider: &DynProvider) -> Vec<QuoterInstance> {
        let mut quoters = Vec::new();
        for tracker in &self.fixed {
            quoters.push(QuoterInstance::Fixed(tracker.clone()));
        }

        if let Some(uniswap_v2_config) = &self.uniswap_v2 {
            for uni_quoters in uniswap_v2_config.pairs.iter() {
                let quoter = UniswapV2Quoter::from_selector(provider, uni_quoters.clone()).await;
                quoters.push(QuoterInstance::UniswapV2(quoter));
            }
        }

        if let Some(uniswap_v3_config) = &self.uniswap_v3 {
            for uni_quoters in uniswap_v3_config.pools.iter() {
                let quoter = UniswapV3Quoter::from_selector(provider, uni_quoters.clone()).await;
                quoters.push(QuoterInstance::UniswapV3(quoter));
            }
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
