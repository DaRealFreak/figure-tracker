use core::fmt;
use std::error::Error;

use chrono::{DateTime, Utc};
use rusqlite::params;
use serde::export::Formatter;

use crate::conditions::ConditionType;
use crate::currency::SupportedCurrency;
use crate::database::conditions::Condition;
use crate::database::items::{Item, ItemConditions};
use crate::database::Database;

#[derive(Clone, Debug)]
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

    /// retrieve the taxed total of the price
    pub fn get_converted_taxed(&self) -> f64 {
        // taxed amount without shipping costs
        self.converted_price * (1.0 + self.taxes)
    }
}

/// Prices implements all related functionality for prices to interact with the database
pub(crate) trait Prices {
    fn add_price(&self, price: &Price) -> Result<(), Box<dyn Error>>;
    fn get_lowest_price_by_item_id(&self, item_id: i64) -> Result<Option<Price>, Box<dyn Error>>;
    fn get_lowest_price_before_price(&self, price: Price) -> Result<Option<Price>, Box<dyn Error>>;
    fn matches_condition(&self, price: Price, condition: Condition) -> bool;
}

struct NoPriceFoundError {}

impl NoPriceFoundError {
    fn display(&self) -> String {
        "no related price was found in the database".to_string()
    }
}

impl std::fmt::Display for NoPriceFoundError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl std::fmt::Debug for NoPriceFoundError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl std::error::Error for NoPriceFoundError {}

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

    fn get_lowest_price_by_item_id(&self, item_id: i64) -> Result<Option<Price>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, item_id, price, currency, converted_price, converted_currency, taxes,
                    shipping, url, module, condition, tstamp
            FROM prices
            WHERE item_id = ?1
            GROUP BY item_id, tstamp
            ORDER BY converted_price
            LIMIT 1",
        )?;

        let mut price_iter = stmt.query_map(params![item_id], |row| {
            Ok(Price {
                id: row.get(0)?,
                item_id: row.get(1)?,
                price: row.get(2)?,
                currency: row.get(3)?,
                converted_price: row.get(4)?,
                converted_currency: row.get(5)?,
                taxes: row.get(6)?,
                shipping: row.get(7)?,
                url: row.get(8)?,
                module: row.get(9)?,
                condition: row.get(10)?,
                timestamp: row.get(11)?,
            })
        })?;

        match price_iter.next() {
            Some(price) => Ok(Some(price.unwrap())),
            None => Err(Box::from(NoPriceFoundError {})),
        }
    }

    fn get_lowest_price_before_price(&self, price: Price) -> Result<Option<Price>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, item_id, price, currency, converted_price, converted_currency, taxes,
                    shipping, url, module, condition, tstamp
            FROM prices
            WHERE item_id = ?1
                AND tstamp < ?2
            GROUP BY item_id, tstamp
            ORDER BY converted_price
            LIMIT 1",
        )?;

        let mut price_iter = stmt.query_map(params![price.item_id, price.timestamp], |row| {
            Ok(Price {
                id: row.get(0)?,
                item_id: row.get(1)?,
                price: row.get(2)?,
                currency: row.get(3)?,
                converted_price: row.get(4)?,
                converted_currency: row.get(5)?,
                taxes: row.get(6)?,
                shipping: row.get(7)?,
                url: row.get(8)?,
                module: row.get(9)?,
                condition: row.get(10)?,
                timestamp: row.get(11)?,
            })
        })?;

        match price_iter.next() {
            Some(price) => Ok(Some(price.unwrap())),
            None => Err(Box::from(NoPriceFoundError {})),
        }
    }

    /// check if the passed price matches the passed condition and should notify the user
    /// about the price
    ///
    /// depending on the condition type the check changes
    /// ConditionType::BelowPrice -> below static value of the condition
    /// ConditionType::BelowPriceTaxed -> below taxed value of the condition
    /// ConditionType::BelowPriceFull -> below the full value of the condition
    /// ConditionType::LowestPrice -> retrieve the lowest item so far and check if the current item is x amount below the value
    /// ConditionType::BelowPrice -> below static value of the condition
    /// ConditionType::PriceDrop -> retrieve the lowest last item recorded before the current item and check if the price dropped by x percentage compared to it
    fn matches_condition(&self, price: Price, condition: Condition) -> bool {
        match condition.condition_type {
            ConditionType::BelowPrice => price.converted_price < condition.value,
            ConditionType::BelowPriceTaxed => price.get_converted_taxed() < condition.value,
            ConditionType::BelowPriceFull => price.get_converted_total() < condition.value,
            ConditionType::LowestPrice => {
                if let Ok(price_option) = self.get_lowest_price_by_item_id(price.item_id) {
                    match price_option {
                        Some(lowest_price) => {
                            price.converted_price + condition.value < lowest_price.converted_price
                        }
                        None => true,
                    }
                } else {
                    false
                }
            }
            ConditionType::PriceDrop => {
                if let Ok(price_option) = self.get_lowest_price_before_price(price.clone()) {
                    match price_option {
                        Some(previous_price) => {
                            previous_price.converted_price * (1.0 - condition.value)
                                > price.converted_price
                        }
                        None => false,
                    }
                } else {
                    false
                }
            }
        }
    }
}
