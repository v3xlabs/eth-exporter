use alloy::primitives::Address;
use alloy::providers::ProviderBuilder;
use alloy::sol;
use poem::listener::TcpListener;
use poem::web::Data;
use poem::{get, handler, EndpointExt as _, Route, Server};
use prometheus::{Encoder, TextEncoder};
use reqwest::Url;
use state::AppState;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

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

async fn update_metrics(state: &Arc<AppState>) {
    let rpc_url: Url = env::var("RPC_URL").unwrap().parse().unwrap();
    let provider = ProviderBuilder::new().on_http(rpc_url);
    let provider_arc = Arc::new(provider);

    for wallet in &state.wallets {
        let provider_arc = provider_arc.clone();

        for erc20_address in &state.erc20s {
            let provider_arc = provider_arc.clone();

            let erc20 = ERC20::new(*erc20_address, provider_arc);
            let balance = erc20.balanceOf(*wallet).call().await.unwrap();
            let balance = balance._0;

            let name = erc20.name().call().await.unwrap();
            let name = name._0;

            println!("{}: {}", name, balance);

            let balance: u64 = balance.to_string().parse().unwrap();

            state
                .balance_of
                .with_label_values(&[
                    wallet.to_string().to_lowercase().as_str(),
                    erc20_address.to_string().to_lowercase().as_str(),
                ])
                .set(balance);
        }
    }
}

#[handler]
async fn metrics(Data(state): Data<&Arc<AppState>>) -> String {
    update_metrics(state).await;

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

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
        .unwrap()
}
