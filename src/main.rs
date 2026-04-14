use state::AppState;

mod cache;
mod http;
mod metrics;
mod state;

#[tokio::main]
pub async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt::init();

    let state = AppState::setup().await;

    http::serve(state).await
}
