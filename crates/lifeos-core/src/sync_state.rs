use serde::{
    Serialize,
    Deserialize
};

use uuid::Uuid;

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize
)]
pub struct SyncState {

    pub device_id: Uuid,

     pub last_seen_operation: Option<Uuid>,

    pub last_seen_sequence: i64,
}