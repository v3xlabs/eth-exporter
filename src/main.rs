use alloy::providers::ProviderBuilder;
use alloy::sol;
use async_std::stream::StreamExt;
use async_std::{future, stream};
use futures::join;
use poem::listener::TcpListener;
use poem::web::Data;
use poem::{get, handler, EndpointExt as _, Route, Server};
use prometheus::{Encoder, TextEncoder};
use state::AppState;
use std::sync::Arc;
use std::time::Duration;

mod state;

// erc20 interface
sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq)]
    interface ERC20 {
        function balanceOf(address owner) external view returns (uint256);
        function name() external view returns (string memory);
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    test().await;

    Ok(())
}

async fn update_metrics(state: &Arc<AppState>) -> anyhow::Result<()> {
    for chain in &state.chains {
        let provider = ProviderBuilder::new().on_http(chain.url.clone());
        let provider_arc = Arc::new(provider);

        for erc20_address in &chain.erc20s {
            let erc20 = ERC20::new(*erc20_address, provider_arc.clone());
            let name = erc20.name().call().await?;
            let name = name._0;

            for wallet in &chain.wallets {
                let balance = erc20.balanceOf(*wallet).call().await?;
                let balance = balance._0;

                println!("{}: {}", name, balance);

                let balance: u64 = balance.to_string().parse()?;

                state
                    .balance_of
                    .with_label_values(&[
                        chain.name.as_str(),
                        erc20_address.to_string().to_lowercase().as_str(),
                        wallet.to_string().to_lowercase().as_str(),
                    ])
                    .set(balance);
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

    let state = AppState::new();

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
