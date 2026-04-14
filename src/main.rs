use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use eth_prices::{
    config::Config,
    router::{graph::QuoterGraph, Route},
    token::TokenIdentifier,
};
use poem::{
    get, handler, listener::TcpListener, web::Data, EndpointExt, Route as PoemRoute, Server,
};
use std::{
    collections::{HashMap, HashSet},
    io::Error,
    sync::Arc,
    time::Duration,
};
use tracing::info;

use crate::{cache::PriceCache, metrics::Metrics};

mod cache;
mod metrics;

pub struct ChainState {
    provider: DynProvider,
    router: QuoterGraph,
    routes: Vec<Route>,
}

pub struct AppState {
    #[allow(dead_code)]
    config: Config,
    cache: PriceCache,
    chains: HashMap<String, ChainState>,
    metrics: Metrics,
}

pub async fn setup() -> AppState {
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

#[handler]
fn index() -> String {
    "hello world!".to_string()
}

#[handler]
async fn get_metrics(state: Data<&Arc<AppState>>) -> String {
    // state.metrics.compute(state.as_ref()).await.unwrap()
    state
        .cache
        .get_or_compute(Arc::clone(state.0))
        .await
        .unwrap()
        .metrics
        .clone()
        .to_string()
}

#[handler]
async fn get_all_metrics(state: Data<&Arc<AppState>>) -> String {
    // returns serde json object of cache
    let data = state
        .cache
        .get_or_compute(Arc::clone(state.0))
        .await
        .unwrap();

    serde_json::to_string(&data.snapshot).unwrap()
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let state = setup().await;

    let app = PoemRoute::new()
        .at("/", get(index))
        .at("/metrics", get(get_metrics))
        .at("/json", get(get_all_metrics))
        .data(Arc::new(state));

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
