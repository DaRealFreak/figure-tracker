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
            "CREATE TABLE IF NOT EXISTS accounts
                (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    user        VARCHAR(255)                DEFAULT '',
                    password    VARCHAR(255)                DEFAULT '',
                    module      VARCHAR(255)    NOT NULL,
                    disabled    BOOLEAN         NOT NULL    DEFAULT FALSE
                 )",
            NO_PARAMS,
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tracked_items
                (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    jan         INTEGER NOT NULL    UNIQUE  DEFAULT '0',
                    term        VARCHAR(255)                DEFAULT '',
                    description VARCHAR(255)                DEFAULT '',
                    disabled    BOOLEAN                     DEFAULT FALSE NOT NULL
                )",
            NO_PARAMS,
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS prices
                (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    item_id     INTEGER NOT NULL    REFERENCES tracked_items(id),
                    price       INTEGER NOT NULL    DEFAULT '0',
                    currency    VARCHAR(255)        DEFAULT '',
                    tstamp      TIMESTAMP           DEFAULT CURRENT_TIMESTAMP
                )",
            NO_PARAMS,
        )?;

        Ok(())
    }
}