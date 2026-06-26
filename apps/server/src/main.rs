use std::sync::{Arc, Mutex};

use axum::serve;
use tokio::net::TcpListener;

use lifeos_storage::{
    db::init_db,
    repository::Repository,
};

use lifeos_engine::sync_engine::SyncEngine;

mod handlers;
mod routes;
mod state;

use routes::create_router;
use state::AppState;

#[tokio::main]
async fn main() {

    let conn = init_db("server.db");

    let repository = Repository::new(conn);

    let sync_engine = Arc::new(
        SyncEngine::new(
            Arc::new(
                Mutex::new(repository)
            )
        )
    );

    let state = AppState {

        sync_engine,
    };

    let app = create_router(state);

    let listener =
        TcpListener::bind("127.0.0.1:3000")
            .await
            .unwrap();

    println!("LifeOS Sync Server");
    println!("Listening on http://127.0.0.1:3000");

    serve(listener, app)
        .await
        .unwrap();
}