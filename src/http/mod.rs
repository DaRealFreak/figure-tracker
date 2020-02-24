use std::error::Error;

use yaml_rust::Yaml;

use crate::configuration::Configuration;

/// small extracted function to check if the checked key exists and is not empty
fn is_config_proxy_key_set(config: &Yaml, key: String) -> bool {
    !config["connection"]["proxy"][key.as_str()].is_badvalue()
        && config["connection"]["proxy"][key.as_str()]
            .as_str()
            .is_some()
        && !config["connection"]["proxy"][key.as_str()]
            .as_str()
            .unwrap()
            .is_empty()
}

/// retrieve the reqwest client based on the connection settings from the configuration
pub fn get_client() -> Result<reqwest::blocking::Client, Box<dyn Error>> {
    let config = Configuration::get_configuration()?;
    let mut builder = reqwest::blocking::Client::builder();

    if is_config_proxy_key_set(&config, "host".to_string()) {
        // proxy host is required
        let mut proxy_host = config["connection"]["proxy"]["host"]
            .as_str()
            .unwrap()
            .to_string();

        // proxy port is optional
        if !config["connection"]["proxy"]["port"].is_badvalue()
            && config["connection"]["proxy"]["port"].as_i64().is_some()
        {
            proxy_host = format!(
                "{}:{:?}",
                &proxy_host,
                config["connection"]["proxy"]["port"].as_i64().unwrap()
            );
        }

        // create the proxy
        let mut proxy = reqwest::Proxy::all(proxy_host.as_str())?;

        // if a username is set we also require a password
        if is_config_proxy_key_set(&config, "username".to_string())
            && is_config_proxy_key_set(&config, "password".to_string())
        {
            // update the proxy object to include basic auth headers in the requests
            proxy = proxy.basic_auth(
                config["connection"]["proxy"]["username"].as_str().unwrap(),
                config["connection"]["proxy"]["password"].as_str().unwrap(),
            )
        }

        // add our proxy to the builder
        builder = builder.proxy(proxy);
    }

    // return the built client
    Ok(builder.build()?)
}
