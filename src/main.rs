use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use eth_prices::{
    config::Config,
    router::{graph::QuoterGraph, Route},
    token::TokenIdentifier,
};
use poem::{
    get, handler, http::StatusCode, listener::TcpListener, web::Data, EndpointExt,
    Route as PoemRoute, Server,
};
use std::{
    collections::{HashMap, HashSet},
    io::Error,
    num::ParseIntError,
    sync::Arc,
    time::Duration,
};
use thiserror::Error;
use tracing::{error, info};

use crate::{cache::PriceCache, metrics::Metrics};

mod cache;
mod metrics;

type Result<T> = std::result::Result<T, SetupError>;

#[derive(Debug, Error)]
pub enum SetupError {
    #[error("failed to load config.toml")]
    Config(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("failed to connect to RPC for chain `{chain}`")]
    Provider {
        chain: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("failed to load quoters for chain `{chain}`")]
    Quoters {
        chain: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("no good route found for token `{token}` on chain `{chain}`")]
    MissingRoute { chain: String, token: String },
    #[error("invalid EE_CACHE_DURATION `{value}`")]
    InvalidCacheDuration {
        value: String,
        #[source]
        source: ParseIntError,
    },
}

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

pub async fn setup() -> Result<AppState> {
    let config = Config::load("config.toml")
        .await
        .map_err(|source| SetupError::Config(Box::new(source)))?;
    let mut chains = HashMap::new();

    for (chain_slug, chain_config) in &config.chains {
        let url = chain_config.rpc_url.clone();
        let provider = ProviderBuilder::new()
            .connect(&url)
            .await
            .map_err(|source| SetupError::Provider {
                chain: chain_slug.clone(),
                source: Box::new(source),
            })?
            .erased();

        for token_config in &chain_config.tokens {
            let token_address = token_config.address.clone();
            println!("token: {:?}", token_address);
        }

        let quoters = chain_config
            .quoters
            .clone()
            .all(&provider)
            .await
            .map_err(|source| SetupError::Quoters {
                chain: chain_slug.clone(),
                source: Box::new(source),
            })?;
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

            let route =
                router
                    .compute(token, &token_out)
                    .map_err(|_| SetupError::MissingRoute {
                        chain: chain_slug.clone(),
                        token: format!("{token:?}"),
                    })?;
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
        Ok(duration) => Duration::from_secs(duration.parse().map_err(|source| {
            SetupError::InvalidCacheDuration {
                value: duration.clone(),
                source,
            }
        })?),
        Err(_) => Duration::from_secs(30),
    };

    Ok(AppState {
        cache: PriceCache::new(duration),
        metrics: Metrics::default(),
        config,
        chains,
    })
}

#[handler]
fn index() -> String {
    "hello world!".to_string()
}

#[handler]
async fn get_metrics(state: Data<&Arc<AppState>>) -> poem::Result<String> {
    state
        .cache
        .get_or_compute(Arc::clone(state.0))
        .await
        .map(|metrics| metrics.to_string())
        .map_err(|err| {
            error!(error = %err, "failed to serve metrics");
            poem::Error::from_string(err.to_string(), StatusCode::SERVICE_UNAVAILABLE)
        })
}

#[tokio::main]
pub async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let state = setup().await.map_err(|err| {
        error!(error = %err, "application startup failed");
        Error::other(err.to_string())
    })?;

    let app = PoemRoute::new()
        .at("/", get(index))
        .at("/metrics", get(get_metrics))
        .data(Arc::new(state));

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
