

use std::sync::{Arc, Mutex};

use lifeos_storage::{
    repository::Repository,
    errors::RepositoryError,
};

use lifeos_sync::{
    api::PushRequest,
    changelog::{
        ChangeLog,
        OperationType,
    },
};

use crate::error::SyncError;

pub struct Processor {
    repository: Arc<Mutex<Repository>>,
}

impl Processor {

    pub fn new(
        repository: Arc<Mutex<Repository>>,
    ) -> Self {

        Self { repository }
    }

    // =====================================================
    // PROCESS PUSH REQUEST
    // =====================================================

    pub fn process_push(
        &self,
        request: &PushRequest,
    ) -> Result<(), SyncError> {

        println!("========================");
        println!("Processing Operations");
        println!("========================");

        for change in &request.changes {

            self.process_change(
                request,
                change,
            )?;
        }

        Ok(())
    }

    // =====================================================
    // PROCESS SINGLE CHANGE
    // =====================================================

    fn process_change(
        &self,
        request: &PushRequest,
        change: &ChangeLog,
    ) -> Result<(), SyncError> {

        let repo = self
            .repository
            .lock()
            .unwrap();

        // ---------------------------------
        // Duplicate detection
        // ---------------------------------

        if repo
            .operation_exists(change.operation_id)
            .map_err(|_| SyncError::RepositoryError)?
        {

            println!(
                "Skipping duplicate operation: {}",
                change.operation_id
            );

            return Ok(());
        }

        // ---------------------------------
        // Dispatch operation
        // ---------------------------------

        match change.operation {

            OperationType::Create => {

                self.process_create(
                    &repo,
                    request,
                    change,
                )?;
            }

            OperationType::Update => {

                self.process_update(
                    &repo,
                    request,
                    change,
                )?;
            }

            OperationType::Delete => {

                self.process_delete(
                    &repo,
                    request,
                    change,
                )?;
            }
        }

        Ok(())
    }

    // =====================================================
    // CREATE
    // =====================================================

    fn process_create(
        &self,
        repo: &Repository,
        request: &PushRequest,
        change: &ChangeLog,
    ) -> Result<(), SyncError> {

        println!(
            "CREATE {}",
            change.entity_id
        );

        let entry = request
            .entries
            .iter()
            .find(|entry| entry.id == change.entity_id);

        let entry = match entry {

            Some(entry) => entry,

            None => {

                return Err(
                    SyncError::ValidationError(
                        format!(
                            "Entry {} not found",
                            change.entity_id
                        )
                    )
                );
            }
        };
match repo.apply_remote_create(entry, change) {

    Ok(_) => {
        println!("Entry persisted successfully.");
    }

    Err(e) => {

        println!("================================");
        println!("Repository Error");
        println!("================================");
        println!("{:#?}", e);
        println!("================================");

        return Err(SyncError::RepositoryError);
    }
}

    

        Ok(())
    }

    // =====================================================
    // UPDATE
    // =====================================================

   fn process_update(
    &self,
    repo: &Repository,
    request: &PushRequest,
    change: &ChangeLog,
) -> Result<(), SyncError> {


    println!(
        "UPDATE {}",
        change.entity_id
    );


    let entry = request
        .entries
        .iter()
        .find(|entry| entry.id == change.entity_id);


    let entry = match entry {

        Some(entry) => entry,

        None => {

            return Err(
                SyncError::ValidationError(
                    format!(
                        "Entry {} not found",
                        change.entity_id
                    )
                )
            );
        }
    };


 match repo.apply_remote_update(
    entry,
    change,
) {

    Ok(_) => {

        println!(
            "Remote update applied successfully"
        );

    }

    Err(RepositoryError::VersionConflict) => {

        println!("================================");
        println!("Version Conflict Detected");
        println!("================================");

        return Err(
            SyncError::ConflictDetected
        );
    }

    Err(RepositoryError::EntryNotFound) => {

        println!("================================");
        println!("Entry Not Found");
        println!("================================");

        return Err(
            SyncError::ValidationError(
                format!(
                    "Entry {} does not exist",
                    change.entity_id
                )
            )
        );
    }

    Err(e) => {

        println!("================================");
        println!("Remote Update Failed");
        println!("================================");
        println!("{:#?}", e);
        println!("================================");

        return Err(
            SyncError::RepositoryError
        );
    }
}


    Ok(())
}
    // =====================================================
    // DELETE
    // =====================================================

    fn process_delete(
    &self,
    repo: &Repository,
    _request: &PushRequest,
    change: &ChangeLog,
) -> Result<(), SyncError> {

    println!(
        "DELETE {}",
        change.entity_id
    );

    match repo.apply_remote_delete(change) {

        Ok(_) => {

            println!(
                "Remote delete applied successfully"
            );

            Ok(())
        }

        Err(RepositoryError::EntryNotFound) => {

            println!(
                "Entry already deleted."
            );

            Ok(())
        }

        Err(e) => {

            println!("{:#?}", e);

            Err(
                SyncError::RepositoryError
            )
        }
    }
}
}



// ========================= TESTS ========================= //

#[cfg(test)]
mod tests {

    use super::*;

    use chrono::Utc;
    use rusqlite::Connection;
    use uuid::Uuid;

    use std::sync::{Arc, Mutex};

    use lifeos_storage::repository::Repository;

    use lifeos_sync::{
        api::PushRequest,
        changelog::{
            ChangeLog,
            OperationType,
        },
    };

    use lifeos_core::models::Entry;


    fn create_test_repository() -> Repository {

        let conn =
            Connection::open_in_memory()
            .unwrap();


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

            "
        )
        .unwrap();


        Repository::new(conn)
    }



    #[test]
    fn test_remote_update_success()
    {

        let repository =
            create_test_repository();


        let repository =
            Arc::new(
                Mutex::new(repository)
            );


        let processor =
            Processor::new(
                repository.clone()
            );



        let device_id =
            Uuid::new_v4();


        let entity_id =
            Uuid::new_v4();



        let original =
            Entry {

                id: entity_id,

                version: 1,

                device_id,

                title:
                    "Original".to_string(),

                content:
                    "Version 1".to_string(),

                created_at:
                    Utc::now(),

                updated_at:
                    Utc::now(),
            };



        {
            let repo =
                repository.lock()
                .unwrap();


            repo.create_entry(
                &original
            )
            .unwrap();
        }



        let updated =
            Entry {

                id: entity_id,

                version: 2,

                device_id,

                title:
                    "Updated".to_string(),

                content:
                    "Version 2".to_string(),

                created_at:
                    original.created_at,

                updated_at:
                    Utc::now(),
            };



        let change =
            ChangeLog {

                sequence:0,

                operation_id:
                    Uuid::new_v4(),

                device_id,

                entity_id,

                operation:
                    OperationType::Update,

                timestamp:
                    Utc::now(),
            };



        let request =
            PushRequest {

                device_id,

                changes:
                    vec![change],

                entries:
                    vec![updated],
            };



        processor
            .process_push(&request)
            .unwrap();



        let repo =
            repository.lock()
            .unwrap();



        let result =
            repo.get_entry(
                &entity_id.to_string()
            )
            .unwrap();



        assert_eq!(
            result.title,
            "Updated"
        );


        assert_eq!(
            result.version,
            2
        );



        let logs =
            repo.get_changes()
            .unwrap();



        assert_eq!(
            logs.len(),
            1
        );


        assert_eq!(
            logs[0].operation,
            OperationType::Update
        );
    }


    #[test]
fn test_update_version_conflict() {

    let repository =
        create_test_repository();

    let repository =
        Arc::new(
            Mutex::new(repository)
        );

    let processor =
        Processor::new(
            repository.clone()
        );

    let device_id =
        Uuid::new_v4();

    let entity_id =
        Uuid::new_v4();

    // ---------------------------------
    // Existing entry in database
    // ---------------------------------

    let original =
        Entry {

            id: entity_id,

            version: 5,

            device_id,

            title:
                "Current".to_string(),

            content:
                "Database Version".to_string(),

            created_at:
                Utc::now(),

            updated_at:
                Utc::now(),
        };

    {
        let repo =
            repository
                .lock()
                .unwrap();

        repo.create_entry(&original)
            .unwrap();
    }

    // ---------------------------------
    // Incoming stale update
    // ---------------------------------

    let stale =
        Entry {

            id: entity_id,

            version: 4,

            device_id,

            title:
                "Old Update".to_string(),

            content:
                "Should Not Win".to_string(),

            created_at:
                original.created_at,

            updated_at:
                Utc::now(),
        };

    let change =
        ChangeLog {

            sequence: 0,

            operation_id:
                Uuid::new_v4(),

            device_id,

            entity_id,

            operation:
                OperationType::Update,

            timestamp:
                Utc::now(),
        };

    let request =
        PushRequest {

            device_id,

            changes:
                vec![change],

            entries:
                vec![stale],
        };

    // ---------------------------------
    // Should fail
    // ---------------------------------

    let result =
        processor.process_push(&request);

    assert!(
        matches!(
            result,
            Err(SyncError::ConflictDetected)
        )
    );

    // ---------------------------------
    // Database should be unchanged
    // ---------------------------------

    let repo =
        repository
            .lock()
            .unwrap();

    let current =
        repo.get_entry(
            &entity_id.to_string()
        )
        .unwrap();

    assert_eq!(
        current.version,
        5
    );

    assert_eq!(
        current.title,
        "Current"
    );

    assert_eq!(
        current.content,
        "Database Version"
    );

    // ---------------------------------
    // No changelog should be added
    // ---------------------------------

    let logs =
        repo.get_changes()
            .unwrap();

    assert_eq!(
        logs.len(),
        0
    );
}

#[test]
fn test_remote_delete_success() {

    let repository =
        create_test_repository();

    let repository =
        Arc::new(
            Mutex::new(repository)
        );

    let processor =
        Processor::new(
            repository.clone()
        );

    let device_id =
        Uuid::new_v4();

    let entity_id =
        Uuid::new_v4();

    // ---------------------------------
    // Insert entry into database
    // ---------------------------------

    let entry =
        Entry {

            id: entity_id,

            version: 1,

            device_id,

            title:
                "Delete Me".to_string(),

            content:
                "Temporary".to_string(),

            created_at:
                Utc::now(),

            updated_at:
                Utc::now(),
        };

    {
        let repo =
            repository
                .lock()
                .unwrap();

        repo.create_entry(&entry)
            .unwrap();
    }

    // ---------------------------------
    // Create DELETE change
    // ---------------------------------

    let change =
        ChangeLog {

            sequence: 0,

            operation_id:
                Uuid::new_v4(),

            device_id,

            entity_id,

            operation:
                OperationType::Delete,

            timestamp:
                Utc::now(),
        };

    let request =
        PushRequest {

            device_id,

            changes:
                vec![change],

            entries:
                vec![],
        };

    // ---------------------------------
    // Execute DELETE
    // ---------------------------------

    processor
        .process_push(&request)
        .unwrap();

    // ---------------------------------
    // Verify entry no longer exists
    // ---------------------------------

    let repo =
        repository
            .lock()
            .unwrap();

    let result =
        repo.get_entry(
            &entity_id.to_string()
        );

    assert!(
        result.is_err()
    );

    // ---------------------------------
    // Verify changelog recorded
    // ---------------------------------

    let logs =
        repo.get_changes()
            .unwrap();

    assert_eq!(
        logs.len(),
        1
    );

    assert_eq!(
        logs[0].operation,
        OperationType::Delete
    );

    assert_eq!(
        logs[0].entity_id,
        entity_id
    );
}

}


