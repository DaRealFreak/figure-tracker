use std::error::Error;

use rusqlite::params;

use crate::conditions::ConditionType;
use crate::database::items::ItemConditions;
use crate::database::Database;

#[derive(Clone)]
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
}
