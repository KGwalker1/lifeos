use rusqlite::Connection;

pub fn init_db() -> Connection {

    let conn =
        Connection::open("lifeos.db")
        .unwrap();

    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS entries (
        
        id TEXT PRIMARY KEY,

        version INTEGER NOT NULL,

        device_id TEXT NOT NULL,

        title TEXT NOT NULL,

        content TEXT NOT NULL,

        created_at TEXT NOT NULL,

        updated_at TEXT NOT NULL
        )
        ",
        [],
    )
    .unwrap();

    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS changelog (

            operation_id TEXT PRIMARY KEY,

            device_id TEXT NOT NULL,

            entity_id TEXT NOT NULL,

            operation TEXT NOT NULL,

            timestamp TEXT NOT NULL
        )
        ",
        [],
    )
    .unwrap();

    // Ensure existing databases get the new columns when upgrading
    let _ = conn.execute(
        "ALTER TABLE entries ADD COLUMN device_id TEXT NOT NULL DEFAULT ''",
        [],
    );

    let _ = conn.execute(
        "ALTER TABLE changelog ADD COLUMN device_id TEXT NOT NULL DEFAULT ''",
        [],
    );

    conn
}