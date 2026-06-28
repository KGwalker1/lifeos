use std::sync::{Arc, Mutex};

use lifeos_storage::repository::Repository;

use lifeos_sync::api::{
    PullRequest,
    PullResponse,
};

use crate::error::SyncError;

pub struct PullProcessor {
    repository: Arc<Mutex<Repository>>,
}

impl PullProcessor {

    pub fn new(
        repository: Arc<Mutex<Repository>>,
    ) -> Self {

        Self { repository }
    }

    pub fn process_pull(
        &self,
        request: &PullRequest,
    ) -> Result<PullResponse, SyncError> {

        println!("========================");
        println!("Processing Pull Request");
        println!("========================");

        let repo =
            self.repository
                .lock()
                .unwrap();

        // Fetch all changes after the client's last sync.
        let changes =
            repo.get_changes_after(
                request.last_seen_sequence,
            )
            .map_err(|_| SyncError::RepositoryError)?;

        // Fetch the corresponding entries.
        let entries =
            repo.get_entries_for_changes(
                &changes,
            )
            .map_err(|_| SyncError::RepositoryError)?;

        // Determine the newest sequence.
        let latest_sequence =
            changes
                .last()
                .map(|c| c.sequence)
                .unwrap_or(request.last_seen_sequence);

        Ok(
            PullResponse {

                entries,

                changes,

                latest_sequence,
            }
        )
    }
    
}
#[cfg(test)]
mod tests {

    use super::*;

    use chrono::Utc;
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    use lifeos_core::models::Entry;
    use lifeos_storage::repository::Repository;
    use lifeos_sync::{
        api::PullRequest,
        changelog::OperationType,
    };

    fn create_test_repository() -> Repository {

        let conn = Connection::open_in_memory().unwrap();

        conn.execute_batch(
            "
            CREATE TABLE entries (
                id TEXT PRIMARY KEY,
                version INTEGER,
                device_id TEXT,
                title TEXT,
                content TEXT,
                created_at TEXT,
                updated_at TEXT
            );

            CREATE TABLE changelog (
                sequence INTEGER PRIMARY KEY AUTOINCREMENT,
                operation_id TEXT UNIQUE,
                device_id TEXT,
                entity_id TEXT,
                operation TEXT,
                timestamp TEXT
            );

            CREATE TABLE sync_state (
                device_id TEXT PRIMARY KEY,
                last_seen_operation TEXT,
                last_seen_sequence INTEGER
            );
            ",
        )
        .unwrap();

        Repository::new(conn)
    }

    #[test]
    fn test_pull_returns_incremental_changes() {

        let repository =
            Arc::new(Mutex::new(create_test_repository()));

        let processor =
            PullProcessor::new(repository.clone());

        let device_id = Uuid::new_v4();

        // -----------------------------
        // Entry 1
        // -----------------------------

        let entry1 = Entry {

            id: Uuid::new_v4(),

            version: 1,

            device_id,

            title: "First".into(),

            content: "Hello".into(),

            created_at: Utc::now(),

            updated_at: Utc::now(),
        };

        {
            let repo = repository.lock().unwrap();

            repo.create_entry_with_log(
                &entry1,
                device_id,
            )
            .unwrap();
        }

        // -----------------------------
        // Entry 2
        // -----------------------------

        let entry2 = Entry {

            id: Uuid::new_v4(),

            version: 1,

            device_id,

            title: "Second".into(),

            content: "World".into(),

            created_at: Utc::now(),

            updated_at: Utc::now(),
        };

        {
            let repo = repository.lock().unwrap();

            repo.create_entry_with_log(
                &entry2,
                device_id,
            )
            .unwrap();
        }

        // ---------------------------------
        // Pull after sequence 1
        // ---------------------------------

        let request = PullRequest {

            device_id,

            last_seen_sequence: 1,
        };

        let response =
            processor
                .process_pull(&request)
                .unwrap();

        // Should only receive sequence 2

        assert_eq!(
            response.changes.len(),
            1
        );

        assert_eq!(
            response.changes[0].sequence,
            2
        );

        assert_eq!(
            response.changes[0].operation,
            OperationType::Create
        );

        // Only the second entry should be returned

        assert_eq!(
            response.entries.len(),
            1
        );

        assert_eq!(
            response.entries[0].title,
            "Second"
        );

        assert_eq!(
            response.latest_sequence,
            2
        );
    }
}