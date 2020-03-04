use std::error::Error;

use crate::http::get_client;

mod base;

#[derive(Clone)]
pub(crate) struct Rakuten {
    client: reqwest::blocking::Client,
}

impl Rakuten {
    /// create new instance of Rakuten
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Rakuten {
            client: get_client()?,
        })
    }

    pub fn get_module_key() -> String {
        "rakuten.co.jp".to_string()
    }
}
