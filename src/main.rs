extern crate chrono;
extern crate env_logger;
#[macro_use]
extern crate log;

use std::error::Error;
use std::io::Write;
use std::process;

use chrono::Local;
use clap::Clap;
use env_logger::Builder;
use log::LevelFilter;

use crate::database::Database;

mod database;

/// Main application for figure tracker
struct FigureTracker {
    options: FigureTrackerOptions,
    db: Option<Database>,
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
    /// URLs for the item to add
    input: Vec<String>,
}

/// main implementation of the figure tracker
impl FigureTracker {
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

    /// open or create the requested SQLite database file, exits if an error occurred
    pub fn open_database(&mut self) {
        let db = database::Database::open("figure_tracker.db");
        if let Err(db) = db {
            error!("couldn't open database (err: {})", db.description());
            process::exit(1)
        }

        self.db = Option::from(db.unwrap());
    }

    /// main entry point, here the CLI options are parsed
    pub fn execute(&mut self) {
        self.initialize_logger();
        self.open_database();

        match &self.options.subcmd {
            SubCommand::Add(t) => {
                match &t.subcmd {
                    AddItemSubCommand::AddItem(item) => {
                        info!("adding item to the database: {:?}", item.input);
                    }
                }
            }
            _ => {
                info!("test")
            }
        }
    }
}

fn main() {
    FigureTracker {
        options: FigureTrackerOptions::parse(),
        db: None,
    }.execute();
}