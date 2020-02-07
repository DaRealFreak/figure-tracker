use rusqlite::{Connection, Error};

use crate::database::migrations::Migration;

// expose migrations
pub(crate) mod migrations;

/// Database contains the persisting database connection for all database operations
pub(crate) struct Database {
    pub(crate) conn: Connection
}

/// Database is the entry point for all database operations
/// by creating or opening a local database file
impl Database {
    pub fn open(path: &str) -> Result<Database, Error> {
        let conn = Connection::open(path)?;
        let db = Database { conn };

        // ensure WAL journal mode instead of DELETE journal mode since I had previously problems
        // in multi threaded applications with the DELETE journal mode
        if !db.uses_wal_journal()? {
            db.set_wal_journal_mode()?;
        }

        // create migrations if not already done
        db.create_migrations()?;

        Ok(db)
    }
}
