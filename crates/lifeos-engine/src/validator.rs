use crate::error::SyncError;

use lifeos_sync::api::PushRequest;

pub struct Validator;

impl Validator {

    pub fn validate_push(
        request: &PushRequest,
    ) -> Result<(), SyncError> {

        if request.device_id.is_nil() {
            return Err(SyncError::InvalidDeviceId);
        }

        Ok(())
    }
}