use std::error::Error;

use crate::http::get_client;

mod base;

#[derive(Clone)]
pub(crate) struct AmazonCoJp {
    client: reqwest::blocking::Client,
}

impl AmazonCoJp {
    /// create new instance of AmazonCoJp
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(AmazonCoJp {
            client: get_client()?,
        })
    }

    pub fn get_module_key() -> String {
        "amazon.co.jp".to_string()
    }
}
