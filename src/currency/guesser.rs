use std::collections::BTreeMap;
use std::error::Error;

use num_traits::ToPrimitive;
use ordered_float::OrderedFloat;
use strsim::normalized_levenshtein;

use crate::currency::SupportedCurrency;

/// struct to store our supported currencies retrieved from the ECB
#[derive(Clone)]
pub(crate) struct CurrencyGuesser {
    currencies: BTreeMap<SupportedCurrency, Vec<String>>,
}

/// implementation of the supported currencies struct
impl CurrencyGuesser {
    /// returns the conversion struct with all currencies
    /// supported by the European Central Bank as of the 19th february 2020
    pub(crate) fn new() -> Self {
        CurrencyGuesser {
            currencies: [
                (SupportedCurrency::EUR, vec!["€".to_string()]),
                (SupportedCurrency::USD, vec!["$".to_string()]),
                (
                    SupportedCurrency::JPY,
                    vec!["¥".to_string(), "YEN".to_string()],
                ),
                (SupportedCurrency::BGN, vec!["лв".to_string()]),
                (SupportedCurrency::CZK, vec!["Kč".to_string()]),
                (SupportedCurrency::DKK, vec!["kr".to_string()]),
                (SupportedCurrency::GBP, vec!["£".to_string()]),
                (SupportedCurrency::HUF, vec!["Ft".to_string()]),
                (SupportedCurrency::PLN, vec!["zł".to_string()]),
                (SupportedCurrency::RON, vec!["lei".to_string()]),
                (SupportedCurrency::SEK, vec!["kr".to_string()]),
                (SupportedCurrency::CHF, vec!["Fr".to_string()]),
                (SupportedCurrency::ISK, vec!["kr".to_string()]),
                (SupportedCurrency::NOK, vec!["kr".to_string()]),
                (SupportedCurrency::HRK, vec!["kn".to_string()]),
                // original symbol would be руб, but the 6 is influencing the levenshtein distance
                // if a number is included in the value, so we use just py
                (SupportedCurrency::RUB, vec!["ру".to_string()]),
                (SupportedCurrency::TRY, vec!["₺".to_string()]),
                (
                    SupportedCurrency::AUD,
                    vec!["$".to_string(), "A$".to_string()],
                ),
                (SupportedCurrency::BRL, vec!["R$".to_string()]),
                (
                    SupportedCurrency::CAD,
                    vec!["$".to_string(), "C$".to_string()],
                ),
                (SupportedCurrency::CNY, vec!["¥".to_string()]),
                (
                    SupportedCurrency::HKD,
                    vec!["$".to_string(), "HK$".to_string()],
                ),
                (SupportedCurrency::IDR, vec!["Rp".to_string()]),
                (SupportedCurrency::ILS, vec!["₪".to_string()]),
                (SupportedCurrency::INR, vec!["₹".to_string()]),
                (SupportedCurrency::KRW, vec!["₩".to_string()]),
                (SupportedCurrency::MXN, vec!["$".to_string()]),
                (SupportedCurrency::MYR, vec!["RM".to_string()]),
                (SupportedCurrency::NZD, vec!["$".to_string()]),
                (SupportedCurrency::PHP, vec!["₱".to_string()]),
                (SupportedCurrency::SGD, vec!["$".to_string()]),
                (SupportedCurrency::THB, vec!["฿".to_string()]),
                (SupportedCurrency::ZAR, vec!["R".to_string()]),
            ]
            .iter()
            .cloned()
            .collect(),
        }
    }

    /// check for currency code matches with the option to only check for exact matches
    pub fn guess_currency_from_code(
        &self,
        value: String,
        exact_match: bool,
    ) -> Option<SupportedCurrency> {
        for (currency_code, _) in self.currencies.iter() {
            if exact_match {
                if value.eq(currency_code.to_string().as_str()) {
                    return Some(currency_code.clone());
                }
            } else if value.contains(currency_code.to_string().as_str()) {
                return Some(currency_code.clone());
            }
        }

        None
    }

    /// will check the passed value for supported currency codes and currency symbols
    /// and return the currency code on a match, returns None if no match was found
    pub fn guess_currency(&self, value: String) -> Option<SupportedCurrency> {
        // check currency code matches first since there are no collisions like with the symbol
        if let Some(currency_code) = self.guess_currency_from_code(value.clone(), false) {
            return Some(currency_code);
        }

        let mut similarities = BTreeMap::new();

        // if there was no match with the currency code we will check the symbols too
        for (currency_code, currency_symbols) in self.currencies.iter() {
            let mut similarity: f64 = 0.0;
            for possible_usage in currency_symbols {
                let distance = normalized_levenshtein(value.as_str(), possible_usage.as_str());
                if distance > similarity {
                    similarity = distance;
                }
            }

            // keep order of registration if similarity is equal (especially useful for $ currencies)
            if similarities.get(&OrderedFloat(similarity)).is_none() {
                similarities.insert(OrderedFloat(similarity), currency_code.clone());
            }
        }

        if let Some(highest_similarity) = similarities.iter().max() {
            return if highest_similarity.0.eq(&OrderedFloat(0.0)) {
                None
            } else {
                Some(highest_similarity.1.clone())
            };
        }

        // nothing found
        None
    }

    /// retrieve the numerical value of the passed amount
    pub fn get_currency_value(value: String) -> Result<f64, Box<dyn Error>> {
        let currency = currency::Currency::from_str(value.as_str())?;
        Ok(currency.value().to_f64().unwrap() / 100.0)
    }
}
