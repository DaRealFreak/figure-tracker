use std::collections::BTreeMap;
use std::error::Error;

use serde::Deserialize;

use crate::currency::guesser::CurrencyGuesser;
use crate::currency::SupportedCurrency;
use crate::http::get_client;

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
