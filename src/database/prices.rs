use std::error::Error;

use chrono::{DateTime, Utc};
use rusqlite::params;

use crate::database::items::{Item, ItemConditions};
use crate::database::Database;

#[derive(Clone)]
pub(crate) struct Price {
    pub(crate) id: Option<i64>,
    pub(crate) price: f64,
    pub(crate) url: String,
    pub(crate) currency: String,
    pub(crate) condition: ItemConditions,
    pub(crate) timestamp: DateTime<Utc>,
}

/// Prices implements all related functionality for prices to interact with the database
pub(crate) trait Prices {
    fn add_price(&self, item: Item, price: Price) -> Result<(), Box<dyn Error>>;
}

/// Prices is the implementation of the Prices trait
impl Prices for Database {
    fn add_price(&self, item: Item, price: Price) -> Result<(), Box<dyn Error>> {
        self.conn.execute(
            "INSERT OR IGNORE INTO prices(item_id, price, url, currency, condition, tstamp)
                  VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                item.id.to_string(),
                price.price,
                price.url,
                price.currency,
                price.condition,
                price.timestamp
            ],
        )?;

        Ok(())
    }
}
