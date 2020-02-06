use rusqlite::{Error, NO_PARAMS};

use crate::database::Database;

/// Migration implements the functionality to migrate the database to the required structure
pub trait Migration {
    fn create_migrations(&self) -> Result<(), Error>;
}

/// Migration is the implementation of the Migration trait
impl Migration for Database {
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