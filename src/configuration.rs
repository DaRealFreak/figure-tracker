use std::fs::{read_to_string, File};
use std::io;
use std::io::Write;
use std::path::Path;

use clap::Clap;
use yaml_rust::{Yaml, YamlLoader};

use crate::cli::FigureTrackerOptions;
use crate::currency::{CurrencyGuesser, SupportedCurrency};

pub(crate) struct Configuration {}

impl Configuration {
    /// parses the passed/default configuration file or creates it if it doesn't exist yet
    pub fn get_configuration() -> Result<Yaml, io::Error> {
        let options = FigureTrackerOptions::parse();
        if !Path::new(options.config.as_str()).exists() {
            let bytes = include_bytes!("../default.yaml");

            let mut file = File::create(options.config.as_str())?;
            file.write_all(bytes)?;
        }

        let config_content = read_to_string(options.config.as_str())?;
        Ok(YamlLoader::load_from_str(config_content.as_str()).unwrap()[0].clone())
    }

    /// retrieve the used currency from the configuration file
    /// returns EUR as the default currency if an error occurred during the retrieval
    pub fn get_used_currency() -> SupportedCurrency {
        if let Ok(conf) = Configuration::get_configuration() {
            if !conf["general"]["currency"].is_badvalue() {
                if let Some(used_currency) = CurrencyGuesser::new().guess_currency_from_code(
                    conf["general"]["currency"].as_str().unwrap().to_string(),
                    true,
                ) {
                    return used_currency;
                }
            }
        }

        SupportedCurrency::EUR
    }
}
