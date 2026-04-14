use std::sync::Arc;

use poem::{
    get, handler, listener::TcpListener, web::Data, EndpointExt, Route as PoemRoute, Server,
};

use crate::state::AppState;

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
    let data = state
        .cache
        .get_or_compute(Arc::clone(state.0))
        .await
        .unwrap();

    serde_json::to_string(&data.snapshot).unwrap()
}

pub async fn serve(state: AppState) -> Result<(), std::io::Error> {
    let app = PoemRoute::new()
        .at("/", get(index))
        .at("/metrics", get(get_metrics))
        .at("/json", get(get_all_metrics))
        .data(Arc::new(state));

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}