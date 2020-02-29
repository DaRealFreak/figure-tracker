use std::error::Error;

use chrono::Utc;

use crate::currency::SupportedCurrency;
use crate::database::items::{Item, ItemConditions};
use crate::database::prices::Price;
use crate::modules::amiami::info::Info;
use crate::modules::amiami::AmiAmi;
use crate::modules::{BaseModule, Prices};

impl BaseModule for AmiAmi {
    fn get_module_key(&self) -> String {
        AmiAmi::get_module_key()
    }

    fn get_lowest_prices(&self, item: &Item) -> Result<Prices, Box<dyn Error>> {
        let mut prices = Prices {
            used: None,
            new: None,
        };

        let api_response = Info { inner: self }.search(item.jan.to_string())?;
        for item in api_response.items {
            if item.instock_flg != 1 {
                continue;
            }

            let cond = if item.condition_flg == 0 {
                ItemConditions::New
            } else {
                ItemConditions::Used
            };

            let price = Price {
                id: None,
                price: item.min_price as f64,
                currency: SupportedCurrency::JPY.to_string(),
                converted_price: 0.0,
                converted_currency: "".to_string(),
                url: item.get_figure_url(),
                module: AmiAmi::get_module_key(),
                condition: cond,
                timestamp: Utc::now(),
            };

            match cond {
                ItemConditions::New => prices.new = Some(price),
                ItemConditions::Used => prices.used = Some(price),
            }
        }
        Ok(prices)
    }

    fn matches_url(&self, _url: &str) -> bool {
        unimplemented!("not implemented yet")
    }
}
