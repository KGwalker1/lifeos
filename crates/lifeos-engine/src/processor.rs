

use std::sync::{Arc, Mutex};

use lifeos_storage::repository::Repository;

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
    ){

        Ok(_) => {

            println!(
                "Remote update applied successfully"
            );

        }


      Err(e)=>{

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
        _repo: &Repository,
        _request: &PushRequest,
        change: &ChangeLog,
    ) -> Result<(), SyncError> {

        println!(
            "DELETE {}",
            change.entity_id
        );

        Ok(())
    }
}


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

}


