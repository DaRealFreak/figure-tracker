extern crate chrono;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate yaml_rust;

use std::borrow::BorrowMut;
use std::error::Error;
use std::io::Write;
use std::process;

use chrono::Local;
use clap::Clap;
use env_logger::Builder;
use log::LevelFilter;
use yaml_rust::Yaml;

use crate::cli::*;
use crate::configuration::Configuration;
use crate::database::items::{Item, Items};
use crate::database::prices::Prices;
use crate::database::Database;
use crate::modules::ModulePool;

mod cli;
mod configuration;
mod currency;
mod database;
mod http;
mod modules;

/// Main application for figure tracker
struct FigureTracker {
    options: FigureTrackerOptions,
    module_pool: ModulePool,
    db: Option<Database>,
    config: Option<Yaml>,
}

/// main implementation of the figure tracker
impl FigureTracker {
    /// initializing function parsing the CLI options
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(FigureTracker {
            options: FigureTrackerOptions::parse(),
            module_pool: ModulePool::new()?,
            db: None,
            config: None,
        })
    }

    /// main entry point, here the CLI options are parsed
    pub fn execute(&mut self) {
        self.initialize_logger();

        match Configuration::get_configuration() {
            Ok(config) => {
                self.config = Some(config);
            }
            Err(err) => error!(
                "couldn't parse or create the configuration (err: {:?})",
                err.description()
            ),
        }

        self.open_database();

        match &self.options.subcmd {
            SubCommand::Add(t) => match &t.subcmd {
                AddSubCommand::Item(item) => {
                    self.add_item(item);
                }
                AddSubCommand::Account(account) => {
                    println!(
                        "{:?}, {:?}, {:?}",
                        account.username, account.password, account.url
                    );
                    unimplemented!("not implemented yet")
                }
            },
            SubCommand::Update(t) => match &t.subcmd {
                UpdateSubCommand::Item(_item) => unimplemented!(),
                UpdateSubCommand::Account(_account) => unimplemented!(),
                UpdateSubCommand::Prices(_t) => {
                    self.update_prices();
                }
            },
        }
    }

    /// initializes the logger across the project
    pub fn initialize_logger(&self) {
        // initialize our logger
        let mut logger = Builder::new();
        logger.format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        });

        // set the log level here
        match self.options.verbose {
            0 => {
                logger.filter(None, LevelFilter::Error);
            }
            1 => {
                logger.filter(None, LevelFilter::Warn);
            }
            2 => {
                logger.filter(None, LevelFilter::Info);
            }
            3 | _ => {
                logger.filter(None, LevelFilter::Debug);
            }
        }

        // initialize our logger
        logger.init();

        // small info about which config file was used
        info!("value for config: {}", self.options.config);
    }

    /// open or create the requested SQLite database file, exits if an error occurred
    pub fn open_database(&mut self) {
        if self.config.as_ref().unwrap()["database"]["path"].is_badvalue() {
            error!("path to the database file is not defined");
            process::exit(1)
        }

        let db = database::Database::open("figure_tracker.db");
        if let Err(db) = db {
            error!("couldn't open database (err: {:?})", db.description());
            process::exit(1)
        }

        self.db = Option::from(db.unwrap());
    }

    /// adds the passed items to the database
    pub fn add_item(&self, add_item: &AddItem) {
        add_item.input.iter().for_each(|new_item| {
            match self.db.as_ref().unwrap().add_item(*new_item) {
                Ok(mut item) => {
                    info!("added item to the database: {:?}", item.jan);
                    self.update_info(item.borrow_mut());
                }
                Err(err) => error!(
                    "unable to add item to the database (err: {:?})",
                    err.description()
                ),
            }
        });
    }

    /// update the information of the passed item using the MFC database
    pub fn update_info(&self, item: &mut Item) {
        match self.module_pool.update_info(item) {
            Ok(_) => match self.db.as_ref().unwrap().update_item(item.to_owned()) {
                Ok(_) => {}
                Err(err) => warn!(
                    "unable to update figure information (err: {:?})",
                    err.description()
                ),
            },
            Err(err) => warn!(
                "unable to find figure information (err: {:?})",
                err.description()
            ),
        }
    }

    /// updates the prices of all tracked items
    pub fn update_prices(&self) {
        match self.db.as_ref().unwrap().get_items() {
            Ok(items) => {
                for item in items {
                    let new_prices = self.module_pool.check_item(item.clone());
                    for price in new_prices {
                        match self
                            .db
                            .as_ref()
                            .unwrap()
                            .add_price(item.clone(), price.clone())
                        {
                            Ok(()) => info!(
                                "detected price for {:?} from module: {:?}: price {:?} (condition: {:?})",
                                item.term_en, price.module, price.price, price.condition
                            ),
                            Err(err) => warn!(
                                "unable to add price to the database (err: {:?})",
                                err.description()
                            ),
                        }
                    }
                }
            }
            Err(err) => warn!(
                "unable to retrieve items from the database (err: {:?})",
                err.description()
            ),
        }
    }
}

fn main() {
    if let Ok(mut app) = FigureTracker::new() {
        app.execute();
    }
}
