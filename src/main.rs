use alloy::network::Ethereum;
use alloy::providers::RootProvider;
use async_std::stream;
use async_std::stream::StreamExt;
use futures::join;
use poem::listener::TcpListener;
use poem::web::Data;
use poem::{get, handler, EndpointExt as _, Route, Server};
use prometheus::{Encoder, TextEncoder};
use state::AppState;
use std::sync::Arc;
use std::time::Duration;

mod models;
mod state;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    test().await;

    Ok(())
}

pub type MyProvider = alloy::providers::fillers::FillProvider<alloy::providers::fillers::JoinFill<alloy::providers::Identity, alloy::providers::fillers::JoinFill<alloy::providers::fillers::GasFiller, alloy::providers::fillers::JoinFill<alloy::providers::fillers::BlobGasFiller, alloy::providers::fillers::JoinFill<alloy::providers::fillers::NonceFiller, alloy::providers::fillers::ChainIdFiller>>>>, RootProvider, Ethereum>;

async fn update_metrics(state: &Arc<AppState>) -> anyhow::Result<()> {
    for chain in &state.chains {
        for erc20_address in &chain.erc20s {
            let name = erc20_address.name.lock().await;
            let usd_price = erc20_address.usd_price.lock().await;

            for wallet in &chain.wallets {
                let balance = erc20_address.erc20.balanceOf(*wallet).call().await?;
                let balance = balance._0;
                let decimals = erc20_address.decimals.lock().await;
                let decimals = *decimals as f64;

                let balance: f64 = balance.to_string().parse()?;
                let balance = balance / 10_f64.powf(decimals);

                println!("{}: {}", name, balance);

                state
                    .balance_of
                    .with_label_values(&[
                        chain.name.as_str(),
                        erc20_address.address.to_string().to_lowercase().as_str(),
                        name.as_str(),
                        wallet.to_string().to_lowercase().as_str(),
                    ])
                    .set(balance);

                if *usd_price > 0.0 {
                    let balance_usd = balance * *usd_price;

                        state
                            .balance_of_usd
                            .with_label_values(&[
                                chain.name.as_str(),
                                erc20_address.address.to_string().to_lowercase().as_str(),
                                name.as_str(),
                                wallet.to_string().to_lowercase().as_str(),
                            ])
                        .set(balance_usd);
                }
            }

            if *usd_price > 0.0 {
                state
                    .price_in_usd
                    .with_label_values(&[
                        chain.name.as_str(),
                        erc20_address.address.to_string().to_lowercase().as_str(),
                        name.as_str(),
                    ])
                    .set(*usd_price);
            }
        }
    }

    Ok(())
}

#[handler]
async fn metrics(Data(state): Data<&Arc<AppState>>) -> String {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_families = state.prometheus.gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

pub async fn test() {
    dotenvy::dotenv().ok();

    let state = AppState::new().await;

    let state = Arc::new(state);

    let app = Route::new()
        .at("/metrics", get(metrics))
        .data(state.clone());

    let http = async {
        Server::new(TcpListener::bind("0.0.0.0:3000"))
            .run(app)
            .await
            .unwrap()
    };

    let updater = async {
        update_metrics(&state).await.ok();

        let mut interval = stream::interval(Duration::from_secs(60));
        while (interval.next().await).is_some() {
            update_metrics(&state).await.ok();
        }
    };

    join!(updater, http);
}
