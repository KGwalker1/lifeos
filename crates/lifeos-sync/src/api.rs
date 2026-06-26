use serde::{Deserialize, Serialize};
use uuid::Uuid;

use lifeos_core::models::Entry;
use crate::changelog::ChangeLog;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushRequest {
    pub device_id: Uuid,
    pub entries: Vec<Entry>,
    pub changes: Vec<ChangeLog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResponse {
    pub success: bool,
    pub latest_sequence: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub device_id: Uuid,
    pub last_seen_sequence: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullResponse {
    pub entries: Vec<Entry>,
    pub changes: Vec<ChangeLog>,
    pub latest_sequence: i64,
}