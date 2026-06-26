# LifeOS

# Phase 1 - Local Synchronization Engine

**Version:** 0.1.0

**Status:** ✅ Completed

---

# Overview

Phase 1 establishes the foundation of the LifeOS synchronization engine.

At the end of this phase the application is capable of:

- Creating entries
- Persisting entries in SQLite
- Recording every modification in an append-only ChangeLog
- Tracking synchronization state for every device
- Returning only changes that occurred after a given sequence number
- Preparing the architecture for future multi-device synchronization

No networking is implemented in this phase.

---

# Architecture

```
                +--------------------+
                |      CLI Test      |
                +---------+----------+
                          |
                          |
                +---------v----------+
                |    Repository      |
                +---------+----------+
                          |
      +-------------------+-------------------+
      |                   |                   |
      |                   |                   |
+-----v-----+     +-------v------+    +-------v------+
|  Entries  |     |  ChangeLog   |    |  SyncState   |
+-----------+     +--------------+    +--------------+
                          |
                    SQLite Database
```

---

# Workspace Structure

```
lifeos/

├── apps/
│   └── cli/
│       └── src/
│           └── main.rs
│
├── crates/
│   ├── lifeos-core/
│   │   ├── models.rs
│   │   ├── changelog.rs
│   │   ├── sync_state.rs
│   │   └── lib.rs
│   │
│   ├── lifeos-storage/
│   │   ├── db.rs
│   │   ├── repository.rs
│   │   └── lib.rs
│   │
│   └── lifeos-sync/
│
└── Cargo.toml
```

---

# Data Models

## Entry

Represents a user document.

Fields

- id
- version
- device_id
- title
- content
- created_at
- updated_at

---

## ChangeLog

Represents an immutable synchronization event.

Fields

- sequence
- operation_id
- device_id
- entity_id
- operation
- timestamp

Every change made by any device is stored here.

The table acts as an append-only event log.

---

## SyncState

Stores the synchronization progress of every device.

Fields

- device_id
- last_seen_operation
- last_seen_sequence

This allows future synchronization requests to ask only for changes after a specific sequence number.

---

# Database Schema

## entries

Stores all entries.

## changelog

Stores every operation performed.

Sequence numbers are automatically generated using

```
INTEGER PRIMARY KEY AUTOINCREMENT
```

Sequence numbers must never be modified manually.

---

## sync_state

Stores synchronization checkpoints.

Each device has only one row.

---

# Repository Responsibilities

Repository currently provides

- Create Entry
- Read Entry
- Read All Entries
- Update Entry
- Delete Entry
- Save ChangeLog
- Read ChangeLog
- Read Delta Changes
- Save SyncState
- Read SyncState

---

# Synchronization Flow

```
User creates entry
        │
        ▼
Repository
        │
        ▼
Insert Entry
        │
        ▼
Create ChangeLog
        │
        ▼
Assign Sequence Number
        │
        ▼
Save Event
```

Future synchronization uses

```
Phone

last_seen_sequence = 152

↓

Server

SELECT *

WHERE sequence > 152
```

Only new changes are transferred.

---

# Why Sequence Numbers?

UUIDs uniquely identify operations.

Sequence numbers define chronological order.

Without sequence numbers there is no efficient way to ask

"Give me everything that happened after my last synchronization."

---

# Testing

Phase 1 has been manually verified using the CLI.

Verified functionality

- Entry creation
- Entry retrieval
- ChangeLog creation
- Sequence generation
- Delta retrieval
- SyncState persistence
- SyncState update

Final test output

```
ALL TESTS PASSED
```

---

# Problems Encountered During Development

## 1. Missing version field

Error

```
no field version on Entry
```

Cause

The Entry struct did not yet contain version.

Solution

Added

```rust
pub version: u32
```

---

## 2. rusqlite does not support u64

Error

```
ToSql not implemented for u64
```

Cause

SQLite stores INTEGER as signed integers.

Solution

Use

```rust
i64
```

instead of

```rust
u64
```

for sequence numbers.

Never use unsigned integers with rusqlite.

---

## 3. Database schema mismatch

Error

```
table entries has no column named version
```

Cause

Database created before schema changed.

Solution

Delete

```
lifeos.db
```

or implement migrations.

During development deleting the database is acceptable.

In production use migrations.

---

## 4. ParseError(TooShort)

Error

```
Uuid::parse_str()
```

failed.

Cause

Wrong SQL column index.

Example

```
operation_id
device_id
entity_id
operation
```

Attempted to parse

```
operation
```

as UUID.

Solution

Always verify SQL column ordering.

---

## 5. More than one primary key

Error

```
table changelog has more than one primary key
```

Cause

Both

```
sequence
```

and

```
operation_id
```

were declared PRIMARY KEY.

Solution

Only

```
sequence
```

is PRIMARY KEY.

operation_id should be

```
UNIQUE
```

---

## 6. SQL syntax error near ')'

Cause

Trailing comma before closing parenthesis.

Incorrect

```
timestamp TEXT,

)
```

Correct

```
timestamp TEXT
)
```

---

## 7. Operation parsed as Update

Cause

Typo

```
"reate"
```

instead of

```
"create"
```

Another issue

Saved

```
create
```

Loaded

```
Create
```

String comparison failed.

Solution

Always use lowercase.

---

## 8. InvalidParameterCount

Cause

SQL placeholders

```
?1
?2
```

did not match supplied parameters.

Solution

Ensure parameter count equals placeholder count.

---

## 9. SyncState Option

Originally

```
last_seen_operation
```

used

```
Uuid
```

Later changed to

```
Option<Uuid>
```

Reason

A newly registered device has never synchronized.

Therefore there is no previous operation.

---

# Lessons Learned

SQLite is strongly typed enough that schema changes require careful planning.

Always verify SQL column order.

Never rely on unwrap() for production code.

Use transactions when modifying multiple tables.

Sequence numbers are essential for synchronization.

UUID identifies.

Sequence orders.

Both are required.

---

# Current Limitations

Phase 1 intentionally does NOT include

- Networking
- REST API
- Authentication
- Conflict Resolution
- Encryption
- Compression
- Background Sync
- Media Synchronization

---

# Future Improvements

Repository should be split into

```
repository/

entries.rs

changelog.rs

sync_state.rs
```

Replace unwrap() with proper error handling.

Use SQLite transactions.

Add indexes

```
idx_changelog_sequence

idx_entries_updated_at
```

Implement automated unit tests.

---

# Phase 2 Goals

Phase 2 introduces networking.

Planned work

- Shared API models
- Axum server
- Push endpoint
- Pull endpoint
- Remote change application
- Conflict detection
- Two-device synchronization

---

# Phase 1 Summary

Completed

✅ SQLite Storage

✅ Repository Layer

✅ Entry CRUD

✅ ChangeLog

✅ Event Sourcing

✅ Sequence Numbers

✅ Delta Synchronization

✅ Device Sync State

✅ Manual Integration Testing

Phase 1 establishes the complete local synchronization foundation for LifeOS.
