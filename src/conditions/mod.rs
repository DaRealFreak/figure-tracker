#[derive(Clone, Copy, Debug)]
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

impl ToString for InvalidConditionError {
    fn to_string(&self) -> String {
        self.msg.clone()
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

impl std::str::FromStr for ConditionType {
    type Err = InvalidConditionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
