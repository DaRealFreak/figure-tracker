use std::error::Error;

use crate::database::items::Item;
use crate::modules::amiami::info::Info;
use crate::modules::amiami::AmiAmi;
use crate::modules::{BaseModule, Prices};

impl BaseModule for AmiAmi {
    fn get_module_key(&self) -> String {
        AmiAmi::get_module_key()
    }

    fn get_lowest_prices(&self, item: &Item) -> Result<Prices, Box<dyn Error>> {
        let api_response = Info { inner: self }.search(item.jan.to_string())?;
        Ok(Prices {
            new: None,
            used: None,
        })
    }

    fn matches_url(&self, _url: &str) -> bool {
        unimplemented!("not implemented yet")
    }
}
