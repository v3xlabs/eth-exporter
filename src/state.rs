use std::{env, str::FromStr, sync::Arc};

use alloy::{primitives::Address, providers::ProviderBuilder};
use futures::future::join_all;
use prometheus::{
    core::{AtomicF64, GenericGaugeVec},
    Opts, Registry,
};
use reqwest::Url;

use crate::models::token::IndexableTokenERC20;

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
    // pub balance_of_usd: GenericGaugeVec<AtomicF64>,
    pub chains: Vec<Chain>,
}

impl AppState {
    pub async fn new() -> Self {
        let prometheus = Registry::new();

        let chain_slugs = vec!["eth", "polygon"];

        let mut chains = Vec::new();
        for chain in chain_slugs {
            let url = env::var(format!("{}_RPC_URL", chain.to_uppercase())).ok();
            if url.is_none() {
                println!("No RPC URL for chain {}", chain);
                continue;
            }

            let url = Url::parse(&url.unwrap()).unwrap();

            let provider = ProviderBuilder::new().on_http(url.clone());
            let provider_arc = Arc::new(provider);

            let wallets: Vec<Address> =
                env::var(format!("{}_WALLET_ADDRESSES", chain.to_uppercase()))
                    .unwrap()
                    .split(',')
                    .map(|s| Address::from_str(s.trim_start_matches("0x")).unwrap())
                    .collect();

            let erc20s: Vec<Address> =
                env::var(format!("{}_ERC20_ADDRESSES", chain.to_uppercase()))
                    .unwrap()
                    .split(',')
                    .map(|s| Address::from_str(s.trim_start_matches("0x")).unwrap())
                    .collect();

            let erc20s = join_all(erc20s.iter().map(|addr| IndexableTokenERC20::new(*addr, provider_arc.clone()))).await;

            chains.push(Chain {
                name: chain.to_string(),
                url,
                wallets,
                erc20s,
            });
        }

        let opts = Opts::new("balance_of", "Balance by user by token");
        let balance_of =
            GenericGaugeVec::new(opts, &["chain", "token", "token_name", "user"]).unwrap();
        prometheus.register(Box::new(balance_of.clone())).unwrap();

        Self {
            prometheus,
            balance_of,
            chains,
        }
    }
}
