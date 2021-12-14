use clap::ArgEnum;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};
use std::fmt::Formatter;

#[derive(ArgEnum, Copy, Clone)]
pub(crate) enum ConditionType {
    BelowPrice,
    BelowPriceTaxed,
    BelowPriceFull,
    LowestPrice,
    PriceDrop,
}

#[derive(Debug)]
pub(crate) struct InvalidConditionError {
    msg: String,
}

impl std::fmt::Display for InvalidConditionError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.msg.clone())
    }
}

impl ToString for ConditionType {
    fn to_string(&self) -> String {
        match self {
            ConditionType::BelowPrice => "below_price".to_string(),
            ConditionType::BelowPriceTaxed => "below_price_taxed".to_string(),
            ConditionType::BelowPriceFull => "below_price_full".to_string(),
            ConditionType::LowestPrice => "lowest_price".to_string(),
            ConditionType::PriceDrop => "price_drop".to_string(),
        }
    }
}

impl std::fmt::Debug for ConditionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl ConditionType {
    fn get_from_str(s: &str) -> Result<Self, InvalidConditionError> {
        match s {
            "below_price" => Ok(ConditionType::BelowPrice),
            "below_price_taxed" => Ok(ConditionType::BelowPriceTaxed),
            "below_price_full" => Ok(ConditionType::BelowPriceFull),
            "lowest_price" => Ok(ConditionType::LowestPrice),
            "price_drop" => Ok(ConditionType::PriceDrop),
            _ => Err(InvalidConditionError {
                msg: format!(
                    "{:?} is not a valid condition type, add --help to see the valid options",
                    s,
                ),
            }),
        }
    }
}

/// to only allow specific item conditions
impl FromSql for ConditionType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match ConditionType::get_from_str(value.as_str()?) {
            Ok(condition_type) => Ok(condition_type),
            Err(_) => Err(FromSqlError::InvalidType),
        }
    }
}
