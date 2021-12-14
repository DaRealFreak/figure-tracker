extern crate chrono;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate yaml_rust;

use std::borrow::BorrowMut;
use std::error::Error;
use std::io::Write;
use std::thread::JoinHandle;
use std::{process, thread};

use chrono::{Local, Utc};
use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;
use yaml_rust::Yaml;

use crate::cli::*;
use crate::configuration::Configuration;
use crate::database::conditions::{Condition, Conditions};
use crate::database::items::{Item, ItemConditions, Items};
use crate::database::prices::{Price, Prices};
use crate::database::Database;
use crate::modules::ModulePool;
use crate::notifications::NotificationManager;

mod cli;
mod conditions;
mod configuration;
mod currency;
mod database;
mod http;
mod modules;
mod notifications;

/// Main application for figure tracker
struct FigureTracker {
    options: FigureTrackerOptions,
    module_pool: ModulePool,
    db: Option<Database>,
    config: Option<Yaml>,
    notifications: NotificationManager,
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
            notifications: NotificationManager::new(),
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
                err
            ),
        }

        self.open_database();

        match &self.options.subcmd {
            SubCommand::Add(t) => match &t.subcmd {
                AddSubCommand::Item(item) => {
                    self.add_item(item);
                }
                AddSubCommand::Notification(notification) => {
                    self.add_notification(notification);
                }
            },
            SubCommand::Update(t) => match &t.subcmd {
                UpdateSubCommand::Item(_item) => unimplemented!(),
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
            error!("couldn't open database (err: {:?})", db);
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
                Err(err) => error!("unable to add item to the database (err: {:?})", err),
            }
        });
    }

    /// adds the requested notification to the database if an item linked to the JAN/EAN is found
    pub fn add_notification(&self, add_notification: &AddNotification) {
        add_notification.items.iter().for_each(|jan| {
            match self.db.as_ref().unwrap().get_item(jan.clone()) {
                Ok(item) => {
                    match self.db.as_ref().unwrap().add_condition(Condition::new(
                        add_notification.condition_type,
                        match add_notification.condition {
                            Some(cond) => cond,
                            None => ItemConditions::All,
                        },
                        add_notification.value,
                        item.id,
                    )) {
                        Ok(_) => info!(
                            "successfully added condition for item: {:?}",
                            item.description
                        ),
                        Err(err) => warn!(
                            "unable to add notification condition to the database (err: {:?}",
                            err
                        ),
                    }
                }
                Err(err) => warn!("unable to retrieve item from the database (err: {:?})", err),
            }
        });
    }

    /// update the information of the passed item using the MFC database
    pub fn update_info(&self, item: &mut Item) {
        match self.module_pool.update_info(item) {
            Ok(_) => match self.db.as_ref().unwrap().update_item(&item) {
                Ok(_) => {}
                Err(err) => warn!("unable to update figure information (err: {:?})", err),
            },
            Err(err) => warn!("unable to find figure information (err: {:?})", err),
        }
    }

    /// updates the prices of all tracked items
    pub fn update_prices(&self) {
        match self.db.as_ref().unwrap().get_items() {
            Ok(items) => {
                let mut notification_handles = vec![];
                for item in items {
                    info!(
                        "updating prices for item: {:?} (JAN {})",
                        item.description, item.jan
                    );

                    let new_prices = self.module_pool.check_item(item.clone());
                    let current_time = Utc::now();
                    for mut price in new_prices.clone() {
                        price.timestamp = current_time;
                        if let Err(err) = self.db.as_ref().unwrap().add_price(&price) {
                            warn!("unable to add price to the database (err: {:?})", err)
                        }
                    }

                    for handle in self.check_conditions(item, new_prices) {
                        notification_handles.push(handle);
                    }
                }

                // wait for all notifications before shutting down the application
                for notification_handle in notification_handles {
                    notification_handle
                        .join()
                        .expect("Couldn't join on the associated thread");
                }
            }
            Err(err) => warn!(
                "unable to retrieve items from the database (err: {:?})",
                err
            ),
        }
    }

    /// check the found prices with the currently saved notifications
    pub fn check_conditions(&self, item: Item, prices: Vec<Price>) -> Vec<JoinHandle<()>> {
        let mut handles = vec![];

        for price in prices {
            if let Ok(related_conditions) = self
                .db
                .as_ref()
                .unwrap()
                .get_related_conditions(item.clone())
            {
                for condition in related_conditions {
                    if self
                        .db
                        .as_ref()
                        .unwrap()
                        .matches_condition(price.clone(), condition.clone())
                    {
                        let not = self.notifications.clone();
                        let shared_item = item.clone();
                        let shared_price = price.clone();
                        handles.push(thread::spawn(move || {
                            match not.notify(shared_item, shared_price, condition) {
                                Ok(_) => info!("notified about condition match..."),
                                Err(err) => {
                                    warn!("couldn't notify about condition match (err: {:?})", err)
                                }
                            }
                        }));
                        continue;
                    }
                }
            }
        }

        handles
    }
}

fn main() {
    let app = FigureTracker::new();
    match app {
        Ok(mut app) => {
            app.execute();
        }
        Err(err) => {
            error!("couldn't execute the program successfully (err: {:?})", err);
        }
    }
}
