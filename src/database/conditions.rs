use std::error::Error;

use rusqlite::params;

use crate::conditions::ConditionType;
use crate::database::items::{Item, ItemConditions};
use crate::database::Database;

#[derive(Clone, Debug)]
pub(crate) struct Condition {
    pub(crate) id: Option<i64>,
    pub(crate) item_id: i64,
    pub(crate) condition_type: ConditionType,
    pub(crate) item_condition: Option<ItemConditions>,
    pub(crate) value: f64,
    pub(crate) disabled: bool,
}

impl Condition {
    pub fn new(
        notification_type: ConditionType,
        condition: Option<ItemConditions>,
        value: f64,
        item_id: i64,
    ) -> Self {
        Condition {
            id: None,
            item_id,
            condition_type: notification_type,
            item_condition: condition,
            value,
            disabled: false,
        }
    }
}

/// Conditions implements all related functionality for conditions to interact with the database
pub(crate) trait Conditions {
    fn add_condition(&self, condition: Condition) -> Result<(), Box<dyn Error>>;
    fn get_related_conditions(&self, item: Item) -> Result<Vec<Condition>, Box<dyn Error>>;
}

impl Conditions for Database {
    fn add_condition(&self, condition: Condition) -> Result<(), Box<dyn Error>> {
        self.conn.execute(
            "INSERT OR IGNORE INTO conditions(
                    item_id, type, condition, value, disabled
                ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                condition.item_id,
                condition.condition_type.to_string(),
                condition.item_condition,
                format!("{:.2}", condition.value),
                condition.disabled
            ],
        )?;

        Ok(())
    }

    /// retrieve related conditions to the passed item
    fn get_related_conditions(&self, item: Item) -> Result<Vec<Condition>, Box<dyn Error>> {
        let mut conditions = vec![];

        let mut stmt = self.conn.prepare(
            "SELECT id, item_id, type, value, condition, disabled
            FROM conditions
            WHERE item_id = ?1",
        )?;

        let conditions_iter = stmt.query_map(params![item.id], |row| {
            Ok(Condition {
                id: row.get(0)?,
                item_id: row.get(1)?,
                condition_type: row.get(2)?,
                value: row.get(3)?,
                item_condition: row.get(4)?,
                disabled: row.get(5)?,
            })
        })?;

        for condition in conditions_iter {
            conditions.push(condition.unwrap());
        }

        Ok(conditions)
    }
}
