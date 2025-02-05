use std::{env, str::FromStr, sync::Arc};

use crate::{models::token::IndexableTokenERC20, MyProvider};
use alloy::{
    network::Ethereum, primitives::Address, providers::ProviderBuilder, transports::http::Http,
};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use futures::future::join_all;
use prometheus::{
    core::{AtomicF64, GenericGaugeVec},
    Opts, Registry,
};
use reqwest::Url;
use serde::Deserialize;

#[derive(Debug)]
pub struct Chain {
    pub name: String,
    pub url: Url,
    pub wallets: Vec<Address>,
    pub erc20s: Vec<IndexableTokenERC20>,
}

pub struct AppState {
    pub prometheus: Registry,
    pub balance_of: GenericGaugeVec<AtomicF64>,
    pub balance_of_usd: GenericGaugeVec<AtomicF64>,
    pub price_in_usd: GenericGaugeVec<AtomicF64>,
    pub chains: Vec<Chain>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ConfigTest {
    pub tokens: std::collections::HashMap<String, ChainConfig>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ChainConfig {
    pub rpc_url: String,
    pub wallets: Vec<String>,
    #[serde(flatten)]
    pub tokens: std::collections::HashMap<String, TokenConfigTest>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct TokenConfigTest {
    pub address: String,
    pub fixed_rate: Option<f64>,
    pub uniswap_v2: Option<String>,
    pub uniswap_v3: Option<String>,
}

impl AppState {
    pub async fn new() -> Self {
        let prometheus = Registry::new();

        // load config test
        let figment = Figment::new().merge(Toml::file("config.toml"));
        let config = figment.extract::<ConfigTest>().unwrap();
        println!("Loaded config: {:#?}", config);

        let mut chains = Vec::new();
        for (chain_slug, chain_config) in config.tokens {
            let url = chain_config.rpc_url;

            let url = Url::parse(&url).unwrap();

            let provider = ProviderBuilder::new().on_http(url.clone());
            let provider_arc = Arc::new(provider);

            let wallets: Vec<Address> = chain_config
                .wallets
                .iter()
                .map(|s| Address::from_str(s).unwrap())
                .collect();

            let erc20s = join_all(
                chain_config
                    .tokens
                    .iter()
                    .map(|(_token_slug, token_config)| {
                        IndexableTokenERC20::new(
                            token_config,
                            chain_slug.clone(),
                            provider_arc.clone(),
                        )
                    }),
            )
            .await;

            chains.push(Chain {
                name: chain_slug,
                url,
                wallets,
                erc20s,
            });
        }

        let opts = Opts::new("balance_of", "Balance by user by token");
        let balance_of =
            GenericGaugeVec::new(opts, &["chain", "token", "token_name", "user"]).unwrap();
        prometheus.register(Box::new(balance_of.clone())).unwrap();

        let opts = Opts::new("balance_of_usd", "Balance by user by token in USD");
        let balance_of_usd =
            GenericGaugeVec::new(opts, &["chain", "token", "token_name", "user"]).unwrap();
        prometheus
            .register(Box::new(balance_of_usd.clone()))
            .unwrap();

        let opts = Opts::new("price_of_usd", "Price in USD");
        let price_in_usd = GenericGaugeVec::new(opts, &["chain", "token", "token_name"]).unwrap();
        prometheus.register(Box::new(price_in_usd.clone())).unwrap();

        Self {
            prometheus,
            balance_of,
            balance_of_usd,
            price_in_usd,
            chains,
        }
    }
}
