extern crate chrono;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate yaml_rust;

use std::error::Error;
use std::fs::{File, read_to_string};
use std::io;
use std::io::Write;
use std::path::Path;
use std::process;

use chrono::Local;
use clap::Clap;
use env_logger::Builder;
use log::LevelFilter;
use yaml_rust::{Yaml, YamlLoader};

use crate::database::Database;
use crate::database::items::Items;

mod database;

/// Main application for figure tracker
struct FigureTracker {
    options: FigureTrackerOptions,
    db: Option<Database>,
    config: Option<Yaml>,
}

/// This application tracks wished items on multiple seller/auction sites
/// and notifies the user about new sales/price drops and price averages
#[derive(Clap)]
#[clap(version = "1.0", author = "DaRealFreak")]
struct FigureTrackerOptions {
    /// Use a custom configuration file.
    #[clap(short = "c", long = "config", default_value = "tracker.yaml")]
    config: String,
    /// A level of verbosity, and can be used multiple times
    #[clap(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: i32,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(name = "add")]
    Add(Add),
    #[clap(name = "update")]
    Update(Add),
}

/// Add an account or item to the database
#[derive(Clap)]
struct Add {
    #[clap(subcommand)]
    subcmd: AddItemSubCommand,
}

#[derive(Clap)]
enum AddItemSubCommand {
    #[clap(name = "item")]
    AddItem(AddItem),
}

/// Add an item to the database
#[derive(Clap)]
struct AddItem {
    /// JAN numbers of the items to add to the tracked items
    input: Vec<u128>,
}

/// main implementation of the figure tracker
impl FigureTracker {
    /// main entry point, here the CLI options are parsed
    pub fn execute(&mut self) {
        self.initialize_logger();

        if let Err(err) = self.parse_configuration() {
            error!("couldn't parse or create the configuration (err: {})", err.description());
            process::exit(1)
        };

        self.open_database();

        match &self.options.subcmd {
            SubCommand::Add(t) => {
                match &t.subcmd {
                    AddItemSubCommand::AddItem(item) => {
                        self.add_item(item);
                    }
                }
            }
            _ => {
                info!("test")
            }
        }
    }

    /// initializes the logger across the project
    pub fn initialize_logger(&self) {
        // Gets a value for config if supplied by user, or defaults to "tracker.yaml"
        println!("Value for config: {}", self.options.config);

        // initialize our logger
        let mut logger = Builder::new();
        logger.format(|buf, record| {
            writeln!(buf,
                     "{} [{}] - {}",
                     Local::now().format("%Y-%m-%dT%H:%M:%S"),
                     record.level(),
                     record.args()
            )
        });

        // set the log level here
        match self.options.verbose {
            0 => { logger.filter(None, LevelFilter::Error); },
            1 => { logger.filter(None, LevelFilter::Warn); },
            2 => { logger.filter(None, LevelFilter::Info); },
            3 | _ => { logger.filter(None, LevelFilter::Debug); },
        }

        // initialize our logger
        logger.init();
    }

    /// parses the passed/default configuration file or creates it if it doesn't exist yet
    pub fn parse_configuration(&mut self) -> Result<(), io::Error> {
        if !Path::new(self.options.config.as_str()).exists() {
            let bytes = include_bytes!("../default.yaml");

            let mut file = File::create(self.options.config.as_str())?;
            file.write_all(bytes)?;
        }

        let config_content = read_to_string(self.options.config.as_str())?;
        self.config = Option::from(YamlLoader::load_from_str(config_content.as_str()).unwrap()[0].clone());

        Ok(())
    }

    /// open or create the requested SQLite database file, exits if an error occurred
    pub fn open_database(&mut self) {
        if self.config.as_ref().unwrap()["database"]["path"].is_badvalue() {
            error!("path to the database file is not defined");
            process::exit(1)
        }

        let db = database::Database::open("figure_tracker.db");
        if let Err(db) = db {
            error!("couldn't open database (err: {})", db.description());
            process::exit(1)
        }

        self.db = Option::from(db.unwrap());
    }

    /// adds the passed items to the database
    pub fn add_item(&self, item: &AddItem) {
        item.input.iter().for_each(|new_item| {
            if let Err(err) = self.db.as_ref().unwrap().add_item(new_item) {
                error!("unable to add item to the database (err: {})", err.description());
            } else {
                info!("added item to the database: {:?}", new_item);
            }
        });
    }
}

fn main() {
    FigureTracker {
        options: FigureTrackerOptions::parse(),
        db: None,
        config: None,
    }.execute();
}