use chrono::{DateTime,Utc};
use serde::{Serialize,Deserialize};
use uuid::Uuid;

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Entry {

    pub id: Uuid,

    pub version: i64,

    pub device_id: Uuid,

    pub title: String,

    pub content: String,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,
}