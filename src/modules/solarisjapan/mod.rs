use std::error::Error;

use crate::http::get_client;

mod base;

#[derive(Clone)]
pub(crate) struct SolarisJapan {
    client: reqwest::blocking::Client,
}

impl SolarisJapan {
    /// create new instance of SolarisJapan
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(SolarisJapan {
            client: get_client()?,
        })
    }

    pub fn get_module_key() -> String {
        "solarisjapan.com".to_string()
    }
}
