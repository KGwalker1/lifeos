use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};

use lifeos_sync::api::{
    PushRequest,
    PushResponse,
    PullRequest,
    PullResponse,
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

pub async fn pull(
    State(state): State<AppState>,
    Json(request): Json<PullRequest>,
) -> impl IntoResponse {

    match state
        .sync_engine
        .process_pull(request)
    {

        Ok(response) => {

            Json(response)
        }

        Err(error) => {

            println!("{:?}", error);

            Json(
                PullResponse {

                    entries: vec![],

                    changes: vec![],

                    latest_sequence: 0,
                }
            )
        }
    }
}