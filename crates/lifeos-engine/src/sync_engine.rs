use std::sync::{Arc, Mutex};

use lifeos_storage::repository::Repository;

use lifeos_sync::api::{
    PushRequest,
    PushResponse,
};

use crate::{
    error::SyncError,
    validator::Validator,
};

use crate::processor::Processor;

pub struct SyncEngine {
    repository: Arc<Mutex<Repository>>,
}

impl SyncEngine {

    pub fn new(
        repository: Arc<Mutex<Repository>>,
    ) -> Self {

        Self {
            repository,
        }
    }

pub fn apply_push(
    &self,
    request: PushRequest,
) -> Result<PushResponse, SyncError> {

    Validator::validate_push(&request)?;

    let processor =
        Processor::new(self.repository.clone());

    processor.process_push(&request)?;

    Ok(PushResponse {

        success: true,

        latest_sequence: 0,
    })
}
}