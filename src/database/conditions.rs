use std::error::Error;

use rusqlite::params;

use crate::conditions::ConditionType;
use crate::database::Database;

#[derive(Clone)]
pub(crate) struct Condition {
    pub(crate) id: Option<i64>,
    pub(crate) condition_type: ConditionType,
    pub(crate) value: f64,
    pub(crate) item_id: i64,
    pub(crate) disabled: bool,
}

impl Condition {
    pub fn new(cond: ConditionType, value: f64, item_id: i64) -> Self {
        Condition {
            id: None,
            condition_type: cond,
            value,
            item_id,
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
                    type, value, item_id, disabled
                ) VALUES (?1, ?2, ?3, ?4)",
            params![
                condition.condition_type.to_string(),
                format!("{:.2}", condition.value),
                condition.item_id,
                condition.disabled
            ],
        )?;

        Ok(())
    }
}
