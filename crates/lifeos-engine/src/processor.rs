use std::sync::{Arc, Mutex};

use lifeos_storage::repository::Repository;

use lifeos_sync::{
    api::PushRequest,
    changelog::OperationType,
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

pub fn process_push(
    &self,
    request: &PushRequest,
) -> Result<(), SyncError> {

    println!("========================");
    println!("Processing Operations");
    println!("========================");

    for change in &request.changes {

        self.process_change(request, change)?;
    }

    Ok(())
}

fn process_change(
    &self,
    request: &PushRequest,
    change: &lifeos_sync::changelog::ChangeLog,
) -> Result<(), SyncError> {

    match change.operation {

        OperationType::Create => {

            self.process_create(request, change)?;
        }

        OperationType::Update => {

            self.process_update(request, change)?;
        }

        OperationType::Delete => {

            self.process_delete(change)?;
        }
    }

    Ok(())

    
}

fn process_create(
    &self,
    request: &PushRequest,
    change: &lifeos_sync::changelog::ChangeLog,
) -> Result<(), SyncError> {

    println!("CREATE {}", change.entity_id);

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

    let repo = self
        .repository
        .lock()
        .unwrap();

repo.apply_remote_create(
    entry,
    change,
)
.map_err(|_| SyncError::RepositoryError)?;

    println!("Entry persisted successfully.");

    Ok(())
}

fn process_update(
    &self,
    _request: &PushRequest,
    change: &lifeos_sync::changelog::ChangeLog,
) -> Result<(), SyncError> {

    println!("UPDATE {}", change.entity_id);

    Ok(())
}

fn process_delete(
    &self,
    change: &lifeos_sync::changelog::ChangeLog,
) -> Result<(), SyncError> {

    println!("DELETE {}", change.entity_id);

    Ok(())
}
}