use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    handlers,
    state::AppState,
};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::health))
        .route("/push", post(handlers::push))
        .with_state(state)
}