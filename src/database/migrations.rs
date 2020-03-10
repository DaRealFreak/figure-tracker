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

        res = self
            .conn
            .query_row("pragma journal_mode", NO_PARAMS, |row| row.get(0));

        Ok(res? == "wal")
    }

    /// sets the WAL journal mode
    fn set_wal_journal_mode(&self) -> Result<(), Error> {
        self.conn.execute_batch("PRAGMA journal_mode=WAL")
    }

    /// create the required tables and alters updated/missing columns on updates
    fn create_migrations(&self) -> Result<(), Error> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tracked_items
                (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    jan         INTEGER NOT NULL UNIQUE DEFAULT '0',
                    term_en     VARCHAR(255)            DEFAULT '',
                    term_jp     VARCHAR(255)            DEFAULT '',
                    description VARCHAR(255)            DEFAULT '',
                    image       VARCHAR(255)            DEFAULT '',
                    disabled    BOOLEAN                 DEFAULT FALSE NOT NULL
                )",
            NO_PARAMS,
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS prices
                (
                    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
                    item_id            INTEGER        NOT NULL REFERENCES tracked_items (id),
                    price              DECIMAL(10, 2) NOT NULL DEFAULT '0',
                    currency           VARCHAR(255)            DEFAULT '',
                    converted_price    DECIMAL(10, 2) NOT NULL DEFAULT '0',
                    converted_currency VARCHAR(255)            DEFAULT '',
                    taxes              DECIMAL(10, 2) NOT NULL DEFAULT '0',
                    shipping           DECIMAL(10, 2) NOT NULL DEFAULT '0',
                    url                VARCHAR(255)            DEFAULT '',
                    module             VARCHAR(255)            DEFAULT '',
                    condition          VARCHAR(255)            DEFAULT '',
                    tstamp             TIMESTAMP               DEFAULT CURRENT_TIMESTAMP
                )",
            NO_PARAMS,
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS conditions
                (
                    id        INTEGER PRIMARY KEY AUTOINCREMENT,
                    item_id   INTEGER        NOT NULL REFERENCES tracked_items (id),
                    type      VARCHAR(255)            DEFAULT '',
                    value     DECIMAL(10, 2) NOT NULL DEFAULT '0',
                    condition VARCHAR(255)            DEFAULT NULL,
                    disabled  BOOLEAN        NOT NULL DEFAULT FALSE
                )",
            NO_PARAMS,
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS notifications
                (
                    id       INTEGER PRIMARY KEY AUTOINCREMENT,
                    type     VARCHAR(255) DEFAULT '',
                    tstamp   TIMESTAMP    DEFAULT CURRENT_TIMESTAMP,
                    item_id  INTEGER NOT NULL REFERENCES tracked_items (id),
                    price_id INTEGER NOT NULL REFERENCES prices (id)
                )",
            NO_PARAMS,
        )?;

        Ok(())
    }
}
