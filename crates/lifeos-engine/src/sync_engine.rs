use std::sync::{Arc, Mutex};

use lifeos_storage::repository::Repository;

use lifeos_sync::api::{
    PushRequest,
    PushResponse,
    PullRequest,
    PullResponse,
};

use crate::{
    error::SyncError,
    validator::Validator,
    processor::Processor,
    pull_processor::PullProcessor,
};

pub struct SyncEngine {

    processor: Processor,

    pull_processor: PullProcessor,
}

impl SyncEngine {

    pub fn new(
        repository: Arc<Mutex<Repository>>,
    ) -> Self {

        let processor =
            Processor::new(
                repository.clone()
            );

        let pull_processor =
            PullProcessor::new(
                repository.clone()
            );

        Self {

            processor,

            pull_processor,
        }
    }

    // =====================================================
    // APPLY PUSH
    // =====================================================

    pub fn apply_push(
        &self,
        request: PushRequest,
    ) -> Result<PushResponse, SyncError> {

        Validator::validate_push(&request)?;

        self.processor
            .process_push(&request)?;

        Ok(
            PushResponse {

                success: true,

                latest_sequence: 0,
            }
        )
    }

    // =====================================================
    // PROCESS PULL
    // =====================================================

    pub fn process_pull(
        &self,
        request: PullRequest,
    ) -> Result<PullResponse, SyncError> {

        self.pull_processor
            .process_pull(&request)
    }
}