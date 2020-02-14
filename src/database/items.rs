use std::error::Error;

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
    fn update_item(&self, item: Item) -> Result<(), Box<dyn Error>>;
}

/// Items is the implementation of the Items trait
impl Items for Database {
    /// retrieve an item from the database based on their JAN number
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

    /// adds an item to the database using only the JAN number
    fn add_item(&self, jan: &i64) -> Result<Item, Box<dyn Error>> {
        self.conn.execute(
            "INSERT OR IGNORE INTO tracked_items(jan)
                  VALUES (?1)",
            params![jan.to_string()],
        )?;

        self.get_item(jan)
    }

    /// synchronize the changes of a mutated item to the database
    fn update_item(&self, item: Item) -> Result<(), Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            "UPDATE tracked_items
                SET jan = ?1, term = ?2, description = ?3, disabled = ?4
                WHERE id = ?5",
        )?;

        stmt.execute(params![
            item.jan,
            item.term,
            item.description,
            item.disabled,
            item.id,
        ])?;

        Ok(())
    }
}
