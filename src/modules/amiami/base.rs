use std::error::Error;

use chrono::Utc;

use crate::database::items::{Item, ItemConditions};
use crate::database::prices::Price;
use crate::modules::amiami::info::Info;
use crate::modules::amiami::AmiAmi;
use crate::modules::{BaseModule, Prices};

impl BaseModule for AmiAmi {
    fn get_module_key(&self) -> String {
        AmiAmi::get_module_key()
    }

    fn get_lowest_prices(&self, item: Item) -> Result<Prices, Box<dyn Error>> {
        let api_response = Info { inner: self }.search(item.jan.to_string())?;
        Ok(Prices {
            new: Option::from(Price {
                id: None,
                price: 10.02,
                url: api_response.get_figure_url(),
                module: self.get_module_key(),
                currency: "¥".to_string(),
                condition: ItemConditions::New,
                timestamp: Utc::now(),
            }),
            used: None,
        })
    }

    fn matches_url(&self, _url: &str) -> bool {
        unimplemented!("not implemented yet")
    }
}
