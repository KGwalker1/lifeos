use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};

use lifeos_sync::api::{
    PushRequest,
    PushResponse,
};

use crate::state::AppState;

pub async fn health() -> impl IntoResponse {

    "LifeOS Sync Server is running"
}

pub async fn push(
    State(state): State<AppState>,
    Json(request): Json<PushRequest>,
) -> impl IntoResponse {

    match state
        .sync_engine
        .apply_push(request)
    {

        Ok(response) => {

            Json(response)
        }

        Err(error) => {

            println!("{:?}", error);

            Json(
                PushResponse {

                    success: false,

                    latest_sequence: 0,
                }
            )
        }
    }
}