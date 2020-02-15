use std::error::Error;

use chrono::NaiveDateTime;

use crate::database::Database;

#[derive(Clone)]
pub(crate) struct Price {
    pub(crate) id: i64,
    pub(crate) price: f64,
    pub(crate) currency: String,
    pub(crate) condition: String,
    pub(crate) timestamp: NaiveDateTime,
}

/// Prices implements all related functionality for prices to interact with the database
pub(crate) trait Prices {
    fn add_price(&self, price: Price) -> Result<(), Box<dyn Error>>;
}

/// Prices is the implementation of the Prices trait
impl Prices for Database {
    fn add_price(&self, price: Price) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }
}
