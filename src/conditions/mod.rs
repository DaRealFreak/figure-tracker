#[derive(Debug)]
pub(crate) enum Conditions {
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

impl std::str::FromStr for Conditions {
    type Err = InvalidConditionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "below_price" => Ok(Conditions::BelowPrice),
            "below_price_taxed" => Ok(Conditions::BelowPriceTaxed),
            "below_price_full" => Ok(Conditions::BelowPriceFull),
            "lowest_price" => Ok(Conditions::LowestPrice),
            "price_drop" => Ok(Conditions::PriceDrop),
            _ => Err(InvalidConditionError {
                msg: format!(
                    "{:?} is not a valid condition type, add --help to see the valid options",
                    s,
                ),
            }),
        }
    }
}
