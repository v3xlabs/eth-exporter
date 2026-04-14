use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use eth_prices::{
    config::Config,
    router::{graph::QuoterGraph, Route},
    token::TokenIdentifier,
};
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};
use tracing::info;

use crate::{cache::PriceCache, metrics::Metrics};

pub struct ChainState {
    pub provider: DynProvider,
    pub router: QuoterGraph,
    pub routes: Vec<Route>,
}

pub struct AppState {
    #[allow(dead_code)]
    pub config: Config,
    pub cache: PriceCache,
    pub chains: HashMap<String, ChainState>,
    pub metrics: Metrics,
}

impl AppState {
    pub async fn setup() -> Self {
        let config = Config::load("config.toml").await.unwrap();
        let mut chains = HashMap::new();
    
        for (chain_slug, chain_config) in &config.chains {
            let url = chain_config.rpc_url.clone();
            let provider = ProviderBuilder::new().connect(&url).await.unwrap().erased();
            for token_config in &chain_config.tokens {
                let token_address = token_config.address.clone();
                println!("token: {:?}", token_address);
            }
            let quoters = chain_config.quoters.clone().all(&provider).await.unwrap();
            let router = QuoterGraph::from_iter(quoters);
    
            let mut all_tokens = HashSet::new();
            for quoter in &router.quoters {
                let (token_in, token_out) = quoter.tokens();
                all_tokens.insert(token_in);
                all_tokens.insert(token_out);
            }
    
            let token_out = TokenIdentifier::Fiat {
                symbol: "usd".to_string(),
            };
            let mut routes = Vec::new();
    
            for token in &all_tokens {
                if token == &token_out {
                    continue;
                }
    
                let route = router
                    .compute(token, &token_out)
                    .expect("Failed to compute route");
                info!("route: {:?}", route);
                routes.push(route);
            }
    
            chains.insert(
                chain_slug.clone(),
                ChainState {
                    provider,
                    router,
                    routes,
                },
            );
        }
    
        let duration = match std::env::var("EE_CACHE_DURATION") {
            Ok(duration) => Duration::from_secs(duration.parse().unwrap()),
            Err(_) => Duration::from_secs(30),
        };
    
        AppState {
            cache: PriceCache::new(duration),
            metrics: Metrics::default(),
            config,
            chains,
        }
    }
}
