use std::error::Error;
use std::str::FromStr;

use rusqlite::params;

use crate::database::Database;

#[derive(Clone)]
pub(crate) struct Item {
    pub(crate) id: i64,
    pub(crate) jan: i64,
    pub(crate) term: String,
    pub(crate) description: String,
    pub(crate) disabled: bool,
}

/// Items implements all related functionality for items to interact with the database
pub(crate) trait Items {
    fn get_item(&self, jan: &i64) -> Result<Item, Box<dyn Error>>;
    fn add_item(&self, jan: &i64) -> Result<Item, Box<dyn Error>>;
}

/// Items is the implementation of the Items trait
impl Items for Database {
    fn get_item(&self, jan: &i64) -> Result<Item, Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, jan, term, description, disabled
            FROM tracked_items
            WHERE jan = ?1",
        )?;

        let mut person_iter = stmt.query_map(params![jan.to_string()], |row| {
            Ok(Item {
                id: row.get(0)?,
                jan: row.get(1)?,
                term: row.get(2)?,
                description: row.get(3)?,
                disabled: row.get(4)?,
            })
        })?;

        Ok(person_iter.next().unwrap()?)
    }

    fn add_item(&self, jan: &i64) -> Result<Item, Box<dyn Error>> {
        self.conn.execute(
            "INSERT OR IGNORE INTO tracked_items(jan)
                  VALUES (?1)",
            params![jan.to_string()],
        )?;

        self.get_item(jan)
    }
}
