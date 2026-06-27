use rusqlite::Error as SqliteError;

#[derive(Debug)]
pub enum RepositoryError {
    Database(SqliteError),

    EntryNotFound,

    VersionConflict,
}

impl From<SqliteError> for RepositoryError {
    fn from(error: SqliteError) -> Self {
        RepositoryError::Database(error)
    }
}