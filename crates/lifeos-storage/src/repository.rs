use rusqlite::{
    params,
    Connection,
    Result,
};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use lifeos_core::models::Entry;

use lifeos_sync::changelog::{
    ChangeLog,
    OperationType,
};

use lifeos_sync::operations::create_event;

use lifeos_core::sync_state::SyncState;

pub struct Repository {
    pub conn: Connection,
}

impl Repository {

    pub fn new(
        conn: Connection,
    ) -> Self {

        Self { conn }
    }

    // =====================================================
    // CREATE ENTRY
    // =====================================================

    pub fn create_entry(
        &self,
        entry: &Entry,
    ) -> Result<()> {

        self.conn.execute(
            "
            INSERT INTO entries
            (
                id,
                version,
                device_id,
                title,
                content,
                created_at,
                updated_at
            )
            VALUES
            (?1,?2,?3,?4,?5,?6,?7)
            ",
            params![
                entry.id.to_string(),
                entry.version,
                entry.device_id.to_string(),
                &entry.title,
                &entry.content,
                entry.created_at.to_rfc3339(),
                entry.updated_at.to_rfc3339()
            ],
        )?;

        Ok(())
    }

    // =====================================================
    // CREATE ENTRY + CHANGELOG
    // =====================================================

    pub fn create_entry_with_log(
        &self,
        entry: &Entry,
        device_id: Uuid,
    ) -> Result<()> {

        self.create_entry(entry)?;

        let change =
            create_event(
                device_id,
                entry.id,
                OperationType::Create,
            );

        self.save_change(&change)?;

        Ok(())
    }

    // =====================================================
    // GET ENTRY
    // =====================================================

    pub fn get_entry(
        &self,
        id: &str,
    ) -> Result<Entry> {

        self.conn.query_row(
            "
            SELECT
                id,
                device_id,
                version,
                title,
                content,
                created_at,
                updated_at
            FROM entries
            WHERE id=?1
            ",
            [id],
            |row| {

                let created_at: String =
                    row.get(5)?;

                let updated_at: String =
                    row.get(6)?;

                Ok(Entry {

                    id:
                        Uuid::parse_str(
                            &row.get::<_, String>(0)?
                        )
                        .unwrap(),

                    version:
                        row.get(2)?,

                    title:
                        row.get(3)?,

                    content:
                        row.get(4)?,

                    device_id:
                        Uuid::parse_str(
                            &row.get::<_, String>(1)?
                        )
                        .unwrap(),

                    created_at:
                        DateTime::parse_from_rfc3339(
                            &created_at
                        )
                        .unwrap()
                        .into(),

                    updated_at:
                        DateTime::parse_from_rfc3339(
                            &updated_at
                        )
                        .unwrap()
                        .into(),
                })
            },
        )
    }

    // =====================================================
    // UPDATE ENTRY
    // =====================================================

    pub fn update_entry(
        &self,
        entry: &mut Entry,
        device_id: Uuid,
    ) -> Result<()> {

        entry.version += 1;

        entry.updated_at = Utc::now();

        self.conn.execute(
            "
            UPDATE entries
            SET
                version=?1,
                title=?2,
                content=?3,
                updated_at=?4
            WHERE id=?5
            ",
            params![
                entry.version,
                &entry.title,
                &entry.content,
                entry.updated_at.to_rfc3339(),
                entry.id.to_string()
            ],
        )?;

        let change =
            create_event(
                device_id,
                entry.id,
                OperationType::Update,
            );

        self.save_change(&change)?;

        Ok(())
    }

    // =====================================================
    // DELETE ENTRY
    // =====================================================

    pub fn delete_entry(
        &self,
        entry_id: Uuid,
        device_id: Uuid,
    ) -> Result<()> {

        self.conn.execute(
            "
            DELETE FROM entries
            WHERE id=?1
            ",
            [entry_id.to_string()],
        )?;

        let change =
            create_event(
                device_id,
                entry_id,
                OperationType::Delete,
            );

        self.save_change(&change)?;

        Ok(())
    }

    // =====================================================
    // SAVE CHANGE
    // =====================================================

    pub fn save_change(
        &self,
        change: &ChangeLog,
    ) -> Result<()> {

        let operation = match change.operation {

            OperationType::Create =>
                "create",

            OperationType::Update =>
                "update",

            OperationType::Delete =>
                "delete",
        };

        self.conn.execute(
            "
            INSERT INTO changelog
            (
                operation_id,
                device_id,
                entity_id,
                operation,
                timestamp
            )
            VALUES
            (?1,?2,?3,?4,?5)
            ",
            params![
                change.operation_id.to_string(),
                change.device_id.to_string(),
                change.entity_id.to_string(),
                operation,
                change.timestamp.to_rfc3339()
            ],
        )?;

        Ok(())
    }

    // =====================================================
    // GET ALL CHANGES
    // =====================================================

    pub fn get_changes(
        &self,
    ) -> Result<Vec<ChangeLog>> {

        let mut stmt =
            self.conn.prepare(
                "
                SELECT
                    sequence,
                    operation_id,
                    device_id,
                    entity_id,
                    operation,
                    timestamp
                FROM changelog
                ORDER BY sequence ASC
                "
            )?;

        let rows =
            stmt.query_map(
                [],
                |row| {

                    let operation: String =
                        row.get(4)?;

                    let timestamp: String =
                        row.get(5)?;

                    Ok(ChangeLog {

                         sequence:
                            row.get(0)?,

                        operation_id:
                            Uuid::parse_str(
                                &row.get::<_, String>(1)?
                            )
                            .unwrap(),

                        device_id:
                            Uuid::parse_str(
                                &row.get::<_, String>(2)?
                            )
                            .unwrap(),

                        entity_id:
                            Uuid::parse_str(
                                &row.get::<_, String>(3)?
                            )
                            .unwrap(),

                        operation:
                            match operation.as_str() {

                                "create" =>
                                    OperationType::Create,

                                "update" =>
                                    OperationType::Update,

                                "delete" =>
                                    OperationType::Delete,

                                _ =>
                                    OperationType::Update,
                            },

                        timestamp:
                            DateTime::parse_from_rfc3339(
                                &timestamp
                            )
                            .unwrap()
                            .into(),
                    })
                },
            )?;

        rows.collect()
    }


    pub fn get_changes_after(
    &self,
    sequence: i64,
) -> Result<Vec<ChangeLog>> {

    let mut stmt =
        self.conn.prepare(
            "
            SELECT
                sequence,
                operation_id,
                device_id,
                entity_id,
                operation,
                timestamp
            FROM changelog
            WHERE sequence > ?1
            ORDER BY sequence ASC
            "
        )?;

    let rows =
        stmt.query_map(
            [sequence],
            |row| {

                let operation: String =
                    row.get(4)?;

                let timestamp: String =
                    row.get(5)?;

                Ok(ChangeLog {

                    sequence:
                        row.get(0)?,

                    operation_id:
                        Uuid::parse_str(
                            &row.get::<_, String>(1)?
                        )
                        .unwrap(),

                    device_id:
                        Uuid::parse_str(
                            &row.get::<_, String>(2)?
                        )
                        .unwrap(),

                    entity_id:
                        Uuid::parse_str(
                            &row.get::<_, String>(3)?
                        )
                        .unwrap(),

                    operation:
                        match operation.as_str() {

    "create" => OperationType::Create,

    "update" => OperationType::Update,

    "delete" => OperationType::Delete,

    _ => OperationType::Update,
},

                    timestamp:
                        DateTime::parse_from_rfc3339(
                            &timestamp
                        )
                        .unwrap()
                        .into(),
                })
            },
        )?;

    rows.collect()
}


    // =====================================================
    // GET ALL ENTRIES
    // =====================================================

    pub fn get_all_entries(
        &self,
    ) -> Result<Vec<Entry>> {

        let mut stmt =
            self.conn.prepare(
                "
                SELECT
                    id,
                    device_id,
                    version,
                    title,
                    content,
                    created_at,
                    updated_at
                FROM entries
                ORDER BY updated_at DESC
                "
            )?;

        let rows =
            stmt.query_map([], |row| {

                let created_at: String =
                    row.get(5)?;

                let updated_at: String =
                    row.get(6)?;

                Ok(Entry {

                    id:
                        Uuid::parse_str(
                            &row.get::<_, String>(0)?
                        )
                        .unwrap(),

                    version:
                        row.get(2)?,

                    title:
                        row.get(3)?,

                    content:
                        row.get(4)?,

                    device_id:
                        Uuid::parse_str(
                            &row.get::<_, String>(1)?
                        )
                        .unwrap(),
                       
                    created_at:
                        DateTime::parse_from_rfc3339(
                            &created_at
                        )
                        .unwrap()
                        .into(),

                    updated_at:
                        DateTime::parse_from_rfc3339(
                            &updated_at
                        )
                        .unwrap()
                        .into(),
                })
            })?;

        rows.collect()
    }


pub fn get_sync_state(
    &self,
    device_id: Uuid,
) -> Result<Option<SyncState>> {

    let mut stmt =
        self.conn.prepare(
            "
            SELECT
                device_id,
                last_seen_operation,
                last_seen_sequence
            FROM sync_state
            WHERE device_id=?1
            "
        )?;

    let mut rows =
        stmt.query([
            device_id.to_string()
        ])?;

    if let Some(row) =
        rows.next()?
    {

        return Ok(Some(
            SyncState {

                device_id:
                    Uuid::parse_str(
                        &row.get::<_, String>(0)?
                    )
                    .unwrap(),

                last_seen_operation:
                    row.get::<_, Option<String>>(1)?
                        .map(|s| Uuid::parse_str(&s).unwrap()),

                last_seen_sequence:
                    row.get(2)?
            }
        ));
    }

    Ok(None)
}

pub fn save_sync_state(
    &self,
    state: &SyncState,
) -> Result<()> {

    self.conn.execute(
        "
        INSERT OR REPLACE
        INTO sync_state
        (
            device_id,
            last_seen_operation,
            last_seen_sequence
        )
        VALUES
        (?1, ?2, ?3)
        ",
        params![
            state.device_id.to_string(),
            state.last_seen_operation
                .map(|op| op.to_string()),
            state.last_seen_sequence,
        ],
    )?;

    Ok(())
}

pub fn operation_exists(
    &self,
    operation_id: Uuid,
) -> rusqlite::Result<bool> {

    let mut stmt = self.conn.prepare(
        "
        SELECT COUNT(*)
        FROM changelog
        WHERE operation_id = ?1
        "
    )?;

    let count: i64 = stmt.query_row(
        [operation_id.to_string()],
        |row| row.get(0),
    )?;

    Ok(count > 0)
}

// =====================================================
// SAVE REMOTE CHANGE
// =====================================================

pub fn save_remote_change(
    tx: &rusqlite::Transaction,
    change: &ChangeLog,
) -> Result<()> {

    let operation = match change.operation {

        OperationType::Create => "create",

        OperationType::Update => "update",

        OperationType::Delete => "delete",
    };


    tx.execute(
        "
        INSERT INTO changelog
        (
            operation_id,
            device_id,
            entity_id,
            operation,
            timestamp
        )
        VALUES
        (?1, ?2, ?3, ?4, ?5)
        ",
        params![

            change.operation_id.to_string(),

            change.device_id.to_string(),

            change.entity_id.to_string(),

            operation,

            change.timestamp.to_rfc3339(),

        ],
    )?;


    Ok(())
}
// =====================================================
// APPLY REMOTE CREATE
// =====================================================

pub fn apply_remote_create(
    &self,
    entry: &Entry,
    change: &ChangeLog,
) -> Result<()> {

    let tx =
        self.conn.unchecked_transaction()?;


    tx.execute(
        "
        INSERT INTO entries
        (
            id,
            version,
            device_id,
            title,
            content,
            created_at,
            updated_at
        )
        VALUES
        (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ",
        params![

            entry.id.to_string(),

            entry.version,

            entry.device_id.to_string(),

            &entry.title,

            &entry.content,

            entry.created_at.to_rfc3339(),

            entry.updated_at.to_rfc3339(),

        ],
    )?;


    Self::save_remote_change(
        &tx,
        change,
    )?;


    tx.commit()?;


    Ok(())
}
// =====================================================
// APPLY REMOTE UPDATE
// =====================================================

pub fn apply_remote_update(
    &self,
    entry: &Entry,
    change: &ChangeLog,
) -> Result<()> {


    let tx =
        self.conn.unchecked_transaction()?;


   let updated =
    tx.execute(
        "
        UPDATE entries
        SET
            version=?1,
            title=?2,
            content=?3,
            updated_at=?4
        WHERE id=?5
        ",
        params![

            entry.version,

            &entry.title,

            &entry.content,

            entry.updated_at.to_rfc3339(),

            entry.id.to_string(),

        ],
    )?;


    if updated == 0 {

    return Err(
        rusqlite::Error::QueryReturnedNoRows
    );
}

    Self::save_remote_change(
        &tx,
        change,
    )?;


    tx.commit()?;


    Ok(())
}
}