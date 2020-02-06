use rusqlite::{Connection, Error};

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
        Ok(Database {
            conn,
        })
    }
}
