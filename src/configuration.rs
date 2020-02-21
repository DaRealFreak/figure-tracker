use std::fs::{read_to_string, File};
use std::io;
use std::io::Write;
use std::path::Path;

use clap::Clap;
use yaml_rust::{Yaml, YamlLoader};

use crate::cli::FigureTrackerOptions;

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
}
