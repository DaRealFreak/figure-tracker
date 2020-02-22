use std::error::Error;

use crate::http::get_client;

mod base;
mod info;

pub(crate) struct AmiAmi {
    client: reqwest::blocking::Client,
}

impl AmiAmi {
    /// create new instance of AmiAmi
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(AmiAmi {
            client: get_client()?,
        })
    }

    pub fn get_module_key() -> String {
        "amiami.com".to_string()
    }
}
