use rusqlite::{Error, params};

use crate::database::Database;

pub(crate) struct Item {
    pub(crate) id: u128,
    pub(crate) jan: u128,
    pub(crate) term: String,
    pub(crate) description: String,
    pub(crate) disabled: bool,
}

/// Items implements all related functionality for items to interact with the database
pub trait Items {
    fn add_item(&self, jan: &u128) -> Result<usize, Error>;
}

/// Items is the implementation of the Items trait
impl Items for Database {
    fn add_item(&self, jan: &u128) -> Result<usize, Error> {
        self.conn.execute(
            "INSERT OR IGNORE INTO tracked_items(jan)
                  VALUES (?1)",
            params![jan.to_string()],
        )
    }
}