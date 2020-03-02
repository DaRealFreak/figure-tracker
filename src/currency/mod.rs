use std::fmt;
use std::fmt::Display;

pub(crate) mod conversion;
pub(crate) mod guesser;

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

#[test]
fn test_currency_conversion() {
    use crate::currency::conversion::CurrencyConversion;

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
    use crate::currency::guesser::CurrencyGuesser;

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
    use crate::currency::guesser::CurrencyGuesser;

    let test_values: Vec<(&str, f64)> = vec![
        // no decimals, no separators
        ("123", 123.0),
        // with decimals, no separators
        ("123.00", 123.0),
        ("123,00", 123.0),
        ("123456.00", 123_456.0),
        ("123456,00", 123_456.0),
        // no decimals, with separators
        ("123.456", 123_456.0),
        ("123,456", 123_456.0),
        ("123.456.789", 123_456_789.0),
        ("123,456,789", 123_456_789.0),
        // with decimals, with separators
        ("123.456,00", 123_456.0),
        ("123,456.00", 123_456.0),
        ("123.456.789,00", 123_456_789.0),
        ("123,456,789.00", 123_456_789.0),
    ];

    for (test_value, expected) in test_values {
        assert_eq!(
            format!(
                "{:.2}",
                CurrencyGuesser::get_currency_value(test_value.to_string()).unwrap()
            ),
            format!("{:.2}", expected)
        );
    }
}
