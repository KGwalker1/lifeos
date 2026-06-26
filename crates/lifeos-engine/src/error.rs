#[derive(Debug)]
pub enum SyncError {

    InvalidDeviceId,

    DuplicateOperation,

    ConflictDetected,

    RepositoryError,

    ValidationError(String),
}