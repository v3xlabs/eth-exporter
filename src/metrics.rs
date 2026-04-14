use std::sync::{atomic::AtomicU64, Arc};

use alloy::providers::Provider;
use eth_prices::token::Token;
use futures::future::try_join_all;
use prometheus_client::{
    encoding::{text::encode, EncodeLabelSet},
    metrics::{family::Family, gauge::Gauge},
    registry::Registry,
};
use tokio::time::Instant;

use crate::{AppState, ChainState};

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct TokenLabels {
    // Use your own enum types to represent label values.
    chain: String,
    // Or just a plain string.
    token: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct Labels {
    chain: String,
}

pub struct Metrics {
    registry: Registry,
    token_price_in_usd: Family<TokenLabels, Gauge<f64, AtomicU64>>,
    block_height: Family<Labels, Gauge<u64, AtomicU64>>,
    chain_duration: Family<Labels, Gauge<u64, AtomicU64>>,
}

impl Default for Metrics {
    fn default() -> Self {
        let mut registry = <Registry>::default();

        let token_price_in_usd = Family::<TokenLabels, Gauge<f64, AtomicU64>>::default();
        registry.register(
            "token_price_usd",
            "Token price in USD",
            token_price_in_usd.clone(),
        );

        let block_height = Family::<Labels, Gauge<u64, AtomicU64>>::default();
        registry.register("block_height", "Block height", block_height.clone());

        let chain_duration = Family::<Labels, Gauge<u64, AtomicU64>>::default();
        registry.register("chain_duration", "Chain duration", chain_duration.clone());

        Self {
            registry,
            token_price_in_usd,
            block_height,
            chain_duration,
        }
    }
}

impl Metrics {
    pub async fn compute(&self, state: Arc<AppState>) -> anyhow::Result<String> {
        let chains = state.chains.iter().collect::<Vec<_>>();
        for (chain_slug, chain) in chains {
            let start_time = Instant::now();
            let block = chain.provider.get_block_number().await?;

            state
                .metrics
                .block_height
                .get_or_create(&Labels {
                    chain: chain_slug.to_string(),
                })
                .set(block);

            try_join_all(
                chain
                    .routes
                    .iter()
                    .map(|route| compute_route_metric(&state, chain_slug, chain, route, block)),
            )
            .await?;

            let duration = start_time.elapsed();
            state
                .metrics
                .chain_duration
                .get_or_create(&Labels {
                    chain: chain_slug.to_string(),
                })
                .set(duration.as_millis() as u64);
        }

        let mut buffer = String::new();
        encode(&mut buffer, &state.metrics.registry).unwrap();

        Ok(buffer)
    }
}

async fn compute_route_metric(
    state: &AppState,
    chain_slug: &str,
    chain: &ChainState,
    route: &eth_prices::router::Route,
    block: u64,
) -> anyhow::Result<()> {
    let token_input = Token::new(route.input_token.clone(), &chain.provider).await?;
    let amount_in = token_input.nominal_amount().await;
    let token_output = route.quote(block, amount_in).await?;

    let rate: i64 = token_output.to_string().parse().unwrap();
    let rate = rate as f64 / 10_f64.powf(6_f64);

    state
        .metrics
        .token_price_in_usd
        .get_or_create(&TokenLabels {
            chain: chain_slug.to_string(),
            token: token_input.symbol.clone(),
        })
        .set(rate);

    Ok(())
}
