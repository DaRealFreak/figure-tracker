use std::error::Error;

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef};
use rusqlite::{params, ToSql};

use crate::database::Database;

#[derive(Clone)]
pub(crate) struct Item {
    pub(crate) id: i64,
    pub(crate) jan: i64,
    pub(crate) term_en: String,
    pub(crate) term_jp: String,
    pub(crate) description: String,
    pub(crate) disabled: bool,
}

/// Available item conditions to request for
#[derive(Clone, Debug)]
pub(crate) enum ItemConditions {
    New,
    Used,
}

/// implementation for the ToSql trait for the rusqlite dependency
/// to only allow specific item conditions
impl ToSql for ItemConditions {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            ItemConditions::New => Ok(ToSqlOutput::from("new")),
            ItemConditions::Used => Ok(ToSqlOutput::from("used")),
        }
    }
}

/// implementation for the FromSql trait for the rusqlite dependency
/// to only allow specific item conditions
impl FromSql for ItemConditions {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| match s {
            "new" => Ok(ItemConditions::New),
            "used" => Ok(ItemConditions::Used),
            _ => Err(FromSqlError::InvalidType),
        })
    }
}

/// Items implements all related functionality for items to interact with the database
pub(crate) trait Items {
    fn get_items(&self) -> Result<Vec<Item>, Box<dyn Error>>;
    fn get_item(&self, jan: i64) -> Result<Item, Box<dyn Error>>;
    fn add_item(&self, jan: i64) -> Result<Item, Box<dyn Error>>;
    fn update_item(&self, item: Item) -> Result<(), Box<dyn Error>>;
}

/// Items is the implementation of the Items trait
impl Items for Database {
    fn get_items(&self) -> Result<Vec<Item>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, jan, term_en, term_jp, description, disabled
             FROM tracked_items
             WHERE disabled = 0",
        )?;

        let res = stmt.query_map(params![], |row| {
            Ok(Item {
                id: row.get(0)?,
                jan: row.get(1)?,
                term_en: row.get(2)?,
                term_jp: row.get(3)?,
                description: row.get(4)?,
                disabled: row.get(5)?,
            })
        })?;

        let mut items: Vec<Item> = vec![];
        for item in res {
            items.push(item.unwrap())
        }

        Ok(items)
    }

    /// retrieve an item from the database based on their JAN number
    fn get_item(&self, jan: i64) -> Result<Item, Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, jan, term_en, term_jp, description, disabled
            FROM tracked_items
            WHERE jan = ?1",
        )?;

        let mut person_iter = stmt.query_map(params![jan.to_string()], |row| {
            Ok(Item {
                id: row.get(0)?,
                jan: row.get(1)?,
                term_en: row.get(2)?,
                term_jp: row.get(3)?,
                description: row.get(4)?,
                disabled: row.get(5)?,
            })
        })?;

        Ok(person_iter.next().unwrap()?)
    }

    /// adds an item to the database using only the JAN number
    fn add_item(&self, jan: i64) -> Result<Item, Box<dyn Error>> {
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
                SET jan = ?1, term_en = ?2, term_jp = ?3, description = ?4, disabled = ?5
                WHERE id = ?6",
        )?;

        stmt.execute(params![
            item.jan,
            item.term_en,
            item.term_jp,
            item.description,
            item.disabled,
            item.id,
        ])?;

        Ok(())
    }
}
