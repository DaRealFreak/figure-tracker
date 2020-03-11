use std::fs::{read_to_string, File};
use std::io;
use std::io::Write;
use std::path::Path;

use clap::Clap;
use yaml_rust::{Yaml, YamlLoader};

use crate::cli::FigureTrackerOptions;
use crate::currency::guesser::CurrencyGuesser;
use crate::currency::SupportedCurrency;

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

    /// retrieve the tax rate to the requested currency
    pub fn get_used_tax_rate(from: SupportedCurrency) -> f64 {
        let mut tax_rate = 0.0;
        if let Ok(conf) = Configuration::get_configuration() {
            if !conf["conversion"]["taxes"][from.to_string().as_str()].is_badvalue() {
                match conf["conversion"]["taxes"][from.to_string().as_str()].clone() {
                    Yaml::Integer(value) => return value as f64,
                    Yaml::Real(value) => {
                        if let Ok(value) = CurrencyGuesser::get_currency_value(value) {
                            tax_rate = value;
                        }
                    }
                    _ => {}
                }
            }
        }

        tax_rate
    }

    /// retrieve the shipping costs based on the passed currency
    pub fn get_shipping(from: SupportedCurrency) -> f64 {
        let mut shipping = 0.0;
        if let Ok(conf) = Configuration::get_configuration() {
            if !conf["conversion"]["shipping"][from.to_string().as_str()].is_badvalue() {
                match conf["conversion"]["shipping"][from.to_string().as_str()].clone() {
                    Yaml::Integer(value) => return value as f64,
                    Yaml::Real(value) => {
                        if let Ok(value) = CurrencyGuesser::get_currency_value(value) {
                            shipping = value;
                        }
                    }
                    _ => {}
                }
            }
        }

        shipping
    }

    /// retrieve the telegram API key is the telegram notification is active and the api key is set
    pub fn get_telegram_api_key() -> Option<String> {
        if let Ok(conf) = Configuration::get_configuration() {
            if !conf["notifications"]["telegram"]["active"].is_badvalue() {
                if let Some(active) = conf["notifications"]["telegram"]["active"].as_bool() {
                    if active
                        && !conf["notifications"]["telegram"]["api_key"].is_badvalue()
                        && conf["notifications"]["telegram"]["api_key"]
                            .as_str()
                            .is_some()
                    {
                        return Some(
                            conf["notifications"]["telegram"]["api_key"]
                                .as_str()
                                .unwrap()
                                .to_string(),
                        );
                    }
                }
            }
        }

        None
    }

    /// retrieve the user ID of the telegram notification option is telegram notification is active
    /// and the user ID is set
    pub fn get_telegram_user_id() -> Option<i64> {
        if let Ok(conf) = Configuration::get_configuration() {
            if !conf["notifications"]["telegram"]["active"].is_badvalue() {
                if let Some(active) = conf["notifications"]["telegram"]["active"].as_bool() {
                    if active
                        && !conf["notifications"]["telegram"]["user_id"].is_badvalue()
                        && conf["notifications"]["telegram"]["user_id"]
                            .as_i64()
                            .is_some()
                    {
                        return conf["notifications"]["telegram"]["user_id"].as_i64();
                    }
                }
            }
        }

        None
    }

    /// retrieve the discord bot token if discord notifications are active and the token is set
    pub fn get_discord_client_token() -> Option<String> {
        if let Ok(conf) = Configuration::get_configuration() {
            if !conf["notifications"]["discord"]["active"].is_badvalue() {
                if let Some(active) = conf["notifications"]["discord"]["active"].as_bool() {
                    if active
                        && !conf["notifications"]["discord"]["client_token"].is_badvalue()
                        && conf["notifications"]["discord"]["client_token"]
                            .as_str()
                            .is_some()
                    {
                        return Some(
                            conf["notifications"]["discord"]["client_token"]
                                .as_str()
                                .unwrap()
                                .to_string(),
                        );
                    }
                }
            }
        }

        None
    }

    /// retrieve the user ID if discord notifications are active and the user ID is set
    pub fn get_discord_user_id() -> Option<i64> {
        if let Ok(conf) = Configuration::get_configuration() {
            if !conf["notifications"]["discord"]["active"].is_badvalue() {
                if let Some(active) = conf["notifications"]["discord"]["active"].as_bool() {
                    if active
                        && !conf["notifications"]["discord"]["user_id"].is_badvalue()
                        && conf["notifications"]["discord"]["user_id"]
                            .as_i64()
                            .is_some()
                    {
                        return conf["notifications"]["discord"]["user_id"].as_i64();
                    }
                }
            }
        }

        None
    }
}
