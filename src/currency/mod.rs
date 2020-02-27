use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fmt::Display;

use num_traits::ToPrimitive;
use ordered_float::OrderedFloat;
use serde::Deserialize;
use strsim::normalized_levenshtein;

use crate::http::get_client;

/// struct to store our supported currencies retrieved from the ECB
#[derive(Clone)]
pub(crate) struct CurrencyGuesser {
    currencies: BTreeMap<SupportedCurrency, Vec<String>>,
}

/// the supported currencies from the ECB
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub(crate) enum SupportedCurrency {
    EUR,
    USD,
    JPY,
    BGN,
    CZK,
    DKK,
    GBP,
    HUF,
    PLN,
    RON,
    SEK,
    CHF,
    ISK,
    NOK,
    HRK,
    RUB,
    TRY,
    AUD,
    BRL,
    CAD,
    CNY,
    HKD,
    IDR,
    ILS,
    INR,
    KRW,
    MXN,
    MYR,
    NZD,
    PHP,
    SGD,
    THB,
    ZAR,
}

/// implement to_string functionality for SupportedCurrency
impl Display for SupportedCurrency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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
                (SupportedCurrency::JPY, vec!["¥".to_string()]),
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

/// structs used for the deserialization of the EUR reference values of the ECB
#[derive(Deserialize)]
struct Envelope {
    #[serde(rename = "Cube")]
    pub cube: Cube,
}

#[derive(Deserialize)]
struct Cube {
    #[serde(rename = "Cube")]
    pub time: CubeTime,
}

#[derive(Deserialize)]
struct CubeTime {
    #[serde(rename = "time")]
    pub date: String,
    #[serde(rename = "Cube")]
    pub currencies: Vec<CubeCurrency>,
}

#[derive(Deserialize)]
struct CubeCurrency {
    pub currency: String,
    pub rate: String,
}

/// struct to use most current exchange rates to convert currencies to one equal currency
#[derive(Clone)]
pub(crate) struct CurrencyConversion {
    pub(crate) exchange_rates: BTreeMap<SupportedCurrency, f64>,
}

///implementation for the currency conversion
impl CurrencyConversion {
    /// retrieve instance of currency conversion with initialized currencies and exchange rates
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(CurrencyConversion {
            exchange_rates: CurrencyConversion::get_conversion_rates()?,
        })
    }

    /// update the conversion rates using the ECB euro reference
    fn get_conversion_rates() -> Result<BTreeMap<SupportedCurrency, f64>, Box<dyn Error>> {
        let mut exchange_rates: BTreeMap<SupportedCurrency, f64> = BTreeMap::new();
        // insert the base currency here which is not in the exchange information
        exchange_rates.insert(SupportedCurrency::EUR, 1.00);

        let res = get_client()?
            .get("https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml")
            .send()?;
        let envelope: Envelope = quick_xml::de::from_str(&res.text()?).unwrap();
        let currency_guesser = CurrencyGuesser::new();

        info!(
            "extracted exchange rates from date: {:?}",
            envelope.cube.time.date
        );

        for currency in envelope.cube.time.currencies.iter() {
            let exchange_rate: f64 = currency.rate.parse()?;
            if let Some(guessed_currency) = currency_guesser
                .guess_currency_from_code(currency.currency.as_str().to_string(), true)
            {
                exchange_rates.insert(guessed_currency.clone(), exchange_rate);
            }
        }

        Ok(exchange_rates)
    }

    /// convert the passed value to the equivalent of the passed to currency
    pub fn convert_price_to(
        &self,
        value: f64,
        from: SupportedCurrency,
        to: SupportedCurrency,
    ) -> f64 {
        value / *self.exchange_rates.get(&from).unwrap() * *self.exchange_rates.get(&to).unwrap()
    }
}

#[test]
fn test_currency_conversion() {
    match CurrencyConversion::new() {
        Ok(converter) => {
            println!(
                "$150 are currently {}€",
                converter.convert_price_to(150.0, SupportedCurrency::USD, SupportedCurrency::EUR)
            );
            println!(
                "150€ are currently ${}",
                converter.convert_price_to(150.0, SupportedCurrency::EUR, SupportedCurrency::USD)
            );
        }
        Err(err) => panic!(err.to_string()),
    }
}

#[test]
fn test_currency_guesses() {
    let guesser = CurrencyGuesser::new();

    let test_values: Vec<(&str, Option<SupportedCurrency>)> = vec![
        ("CAD250.00", Some(SupportedCurrency::CAD)),
        ("C$325.00", Some(SupportedCurrency::CAD)),
        ("350.31€", Some(SupportedCurrency::EUR)),
        ("$159.99", Some(SupportedCurrency::USD)),
        ("150", None),
        ("£420.00", Some(SupportedCurrency::GBP)),
        ("A$180.00", Some(SupportedCurrency::AUD)),
        ("HK$520.00", Some(SupportedCurrency::HKD)),
    ];

    for (test_value, expected_currency) in test_values {
        println!(
            "assuming currency: {:?} for value: {}",
            guesser.guess_currency(test_value.to_string()),
            test_value
        );
        assert_eq!(
            guesser.guess_currency(test_value.to_string()),
            expected_currency
        );
    }
}

#[test]
pub fn test_currency_value_guesses() {
    let test_values = vec![
        // no decimals, no separators
        ("123", 123.0),
        // with decimals, no separators
        ("123.00", 123.0),
        ("123,00", 123.0),
        ("123456.00", 123456.0),
        ("123456,00", 123456.0),
        // no decimals, with separators
        ("123.456", 123456.0),
        ("123,456", 123456.0),
        ("123.456.789", 123456789.0),
        ("123,456,789", 123456789.0),
        // with decimals, with separators
        ("123.456,00", 123456.0),
        ("123,456.00", 123456.0),
        ("123.456.789,00", 123456789.0),
        ("123,456,789.00", 123456789.0),
    ];

    for (test_value, expected) in test_values {
        assert_eq!(
            CurrencyGuesser::get_currency_value(test_value.to_string()).unwrap(),
            expected
        );
    }
}
