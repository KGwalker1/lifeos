use std::sync::Arc;

use lifeos_engine::sync_engine::SyncEngine;

#[derive(Clone)]
pub struct AppState {

    pub sync_engine: Arc<SyncEngine>,
}