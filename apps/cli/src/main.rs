use chrono::Utc;
use uuid::Uuid;

use lifeos_core::models::Entry;
use lifeos_storage::{
    db::init_db,
    repository::Repository,
};

fn main() {

    println!("Starting LifeOS...");

    let conn = init_db();

    let repo = Repository::new(conn);

    let device_id = Uuid::new_v4();

    let entry = Entry {

        id: Uuid::new_v4(),

        version: 1,
        
device_id: device_id,

        title: "First Entry".to_string(),

        content: "Testing sync engine".to_string(),

        created_at: Utc::now(),

        updated_at: Utc::now(),
    };

    repo
        .create_entry_with_log(
            &entry,
            device_id,
        )
        .unwrap();

    println!("Entry Created");

    let loaded = repo
        .get_entry(
            &entry.id.to_string(),
        )
        .unwrap();

    println!(
        "Loaded Entry:\n{:#?}",
        loaded
    );

    let changes = repo
        .get_changes()
        .unwrap();

    println!(
        "Changes Recorded: {}",
        changes.len()
    );

    for change in changes {

        println!("{:#?}", change);
    }
}