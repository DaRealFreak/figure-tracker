use rusqlite::{Error, NO_PARAMS};

use crate::database::Database;

/// Migration implements the functionality to migrate the database to the required structure
pub trait Migration {
    fn uses_wal_journal(&self) -> Result<bool, Error>;
    fn set_wal_journal_mode(&self) -> Result<(), Error>;
    fn create_migrations(&self) -> Result<(), Error>;
}

/// Migration is the implementation of the Migration trait
impl Migration for Database {
    /// check if the current connection is using the WAL journal mode
    fn uses_wal_journal(&self) -> Result<bool, Error> {
        let res: Result<String, Error>;

        res = self.conn.query_row(
            "pragma journal_mode",
            NO_PARAMS,
            |row| row.get(0),
        );

        Ok(res? == "wal")
    }

    /// sets the WAL journal mode
    fn set_wal_journal_mode(&self) -> Result<(), Error> {
        self.conn.execute_batch("PRAGMA journal_mode=WAL")
    }

    /// create the required tables and alters updated/missing columns on updates
    fn create_migrations(&self) -> Result<(), Error> {
        self.conn.execute(
            "create table if not exists cat_colors (
                     id integer primary key,
                     name text not null unique
                 )",
            NO_PARAMS,
        )?;
        self.conn.execute(
            "create table if not exists cats (
                     id integer primary key,
                     name text not null,
                     color_id integer not null references cat_colors(id)
                 )",
            NO_PARAMS,
        )?;

        Ok(())
    }
}