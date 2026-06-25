use chrono::Utc;
use uuid::Uuid;

use crate::changelog::*;

pub fn create_event(
    device_id: Uuid,
    entity_id: Uuid,
    operation: OperationType,
) -> ChangeLog {

    ChangeLog {
    
        sequence: 0,

        operation_id: Uuid::new_v4(),

        device_id,

        entity_id,

        operation,

        timestamp: Utc::now(),
    }
}