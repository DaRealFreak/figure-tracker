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
            "BelowPrice" => Ok(Conditions::BelowPrice),
            "BelowPriceTaxed" => Ok(Conditions::BelowPriceTaxed),
            "BelowPriceFull" => Ok(Conditions::BelowPriceFull),
            "LowestPrice" => Ok(Conditions::LowestPrice),
            "PriceDrop" => Ok(Conditions::PriceDrop),
            _ => Err(InvalidConditionError {
                msg: format!(
                    "{:?} is not a valid condition type, valid types are {:?}, {:?}, {:?}, {:?}, {:?}",
                    s,
                    Conditions::BelowPrice,
                    Conditions::BelowPriceTaxed,
                    Conditions::BelowPriceFull,
                    Conditions::LowestPrice,
                    Conditions::PriceDrop,
                ),
            }),
        }
    }
}
