use chrono::Utc;
use uuid::Uuid;

use lifeos_core::{
    models::Entry,
    sync_state::SyncState,
};

use lifeos_storage::{
    db::init_db,
    repository::Repository,
};

fn main() {

    println!("=================================");
    println!("Starting LifeOS Sync Engine Test");
    println!("=================================");

    let conn = init_db("lifeos.db");

    let repo = Repository::new(conn);

    let device_id = Uuid::new_v4();

    println!("\nDevice ID:");
    println!("{}", device_id);

    // =================================
    // CREATE ENTRY
    // =================================

    let entry = Entry {

        id: Uuid::new_v4(),

        version: 1,

        device_id,

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

    println!("\n✓ Entry Created");

    // =================================
    // LOAD ENTRY
    // =================================

    let loaded =
        repo
            .get_entry(
                &entry.id.to_string(),
            )
            .unwrap();

    println!("\nLoaded Entry:");
    println!("{:#?}", loaded);

    // =================================
    // ALL CHANGES
    // =================================

    let changes =
        repo
            .get_changes()
            .unwrap();

    println!(
        "\nTotal Changes Recorded: {}",
        changes.len()
    );

    for change in &changes {

        println!(
            "\nSequence {}",
            change.sequence
        );

        println!("{:#?}", change);
    }

    // =================================
    // DELTA TEST
    // =================================

    let delta =
        repo
            .get_changes_after(0)
            .unwrap();

    println!(
        "\nDelta After Sequence 0: {}",
        delta.len()
    );

    for change in &delta {

        println!(
            "Delta Sequence {}",
            change.sequence
        );
    }

    // =================================
    // SAVE SYNC STATE
    // =================================

    let last_change =
        changes
            .last()
            .unwrap();

    let sync_state = SyncState {

        device_id,

        last_seen_operation:
            Some(
                last_change.operation_id
            ),

        last_seen_sequence:
            last_change.sequence,
    };

    repo
        .save_sync_state(
            &sync_state
        )
        .unwrap();

    println!("\n✓ Sync State Saved");

    // =================================
    // LOAD SYNC STATE
    // =================================

    let loaded_state =
        repo
            .get_sync_state(
                device_id
            )
            .unwrap();

    println!("\nLoaded Sync State:");

    println!(
        "{:#?}",
        loaded_state
    );

    // =================================
    // UPDATE SYNC STATE
    // =================================

    let updated_state = SyncState {

        device_id,

        last_seen_operation:
            Some(
                Uuid::new_v4()
            ),

        last_seen_sequence: 999,
    };

    repo
        .save_sync_state(
            &updated_state
        )
        .unwrap();

    let reloaded =
        repo
            .get_sync_state(
                device_id
            )
            .unwrap();

    println!("\nUpdated Sync State:");

    println!(
        "{:#?}",
        reloaded
    );

    println!("\n=================================");
    println!("ALL TESTS PASSED");
    println!("=================================");
}