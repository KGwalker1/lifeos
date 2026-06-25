use chrono::{DateTime,Utc};
use serde::{Serialize,Deserialize};
use uuid::Uuid;

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum OperationType {

    Create,

    Update,

    Delete,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ChangeLog {

    pub operation_id: Uuid,

    pub device_id: Uuid,

    pub entity_id: Uuid,

    pub operation: OperationType,

    pub timestamp: DateTime<Utc>,
}