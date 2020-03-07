use std::error::Error;

use chrono::{DateTime, Utc};
use rusqlite::params;

use crate::currency::SupportedCurrency;
use crate::database::items::{Item, ItemConditions};
use crate::database::Database;

#[derive(Clone)]
pub(crate) struct Price {
    pub(crate) id: Option<i64>,
    pub(crate) item_id: i64,
    pub(crate) price: f64,
    pub(crate) currency: String,
    pub(crate) converted_price: f64,
    pub(crate) converted_currency: String,
    pub(crate) taxes: f64,
    pub(crate) shipping: f64,
    pub(crate) url: String,
    pub(crate) module: String,
    pub(crate) condition: ItemConditions,
    pub(crate) timestamp: DateTime<Utc>,
}

impl Price {
    /// retrieve price from only the required attributes
    pub fn new(
        item: Item,
        price: f64,
        currency: SupportedCurrency,
        url: String,
        module: String,
        condition: ItemConditions,
    ) -> Self {
        Price {
            id: None,
            item_id: item.id,
            price,
            currency: currency.to_string(),
            converted_price: 0.0,
            converted_currency: currency.to_string(),
            taxes: 0.0,
            shipping: 0.0,
            url,
            module,
            condition,
            timestamp: Utc::now(),
        }
    }

    /// retrieve the relevant total of the price
    pub fn get_converted_total(&self) -> f64 {
        // shipping costs are normally also taxed
        (self.converted_price + self.shipping) * (1.0 + self.taxes)
    }
}

/// Prices implements all related functionality for prices to interact with the database
pub(crate) trait Prices {
    fn add_price(&self, price: &Price) -> Result<(), Box<dyn Error>>;
}

/// Prices is the implementation of the Prices trait
impl Prices for Database {
    fn add_price(&self, price: &Price) -> Result<(), Box<dyn Error>> {
        self.conn.execute(
            "INSERT OR IGNORE INTO prices(
                    item_id, price, currency, converted_price, converted_currency, taxes,
                    shipping, url, module, condition, tstamp
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                price.item_id,
                format!("{:.2}", price.price),
                price.currency,
                format!("{:.2}", price.converted_price),
                price.converted_currency,
                price.taxes,
                price.shipping,
                price.url,
                price.module,
                price.condition,
                price.timestamp
            ],
        )?;

        Ok(())
    }
}
