extern crate chrono;
extern crate env_logger;
#[macro_use]
extern crate log;

use std::io::Write;

use chrono::Local;
use clap::Clap;
use env_logger::Builder;
use log::LevelFilter;

mod database;

/// This application tracks wished items on multiple seller/auction sites
/// and notifies the user about new sales/price drops and price averages
#[derive(Clap)]
#[clap(version = "1.0", author = "DaRealFreak")]
struct FigureTracker {
    /// Use a custom configuration file.
    #[clap(short = "c", long = "config", default_value = "tracker.yaml")]
    config: String,
    /// A level of verbosity, and can be used multiple times
    #[clap(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: i32,
    #[clap(subcommand)]
    subcmd: AddSubCommand,
}

#[derive(Clap)]
enum AddSubCommand {
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

impl FigureTracker {
    /// initializes the logger across the project
    pub fn initialize_logger(&self) {
        // Gets a value for config if supplied by user, or defaults to "tracker.yaml"
        println!("Value for config: {}", self.config);

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
        match self.verbose {
            0 => { logger.filter(None, LevelFilter::Error); },
            1 => { logger.filter(None, LevelFilter::Warn); },
            2 => { logger.filter(None, LevelFilter::Info); },
            3 | _ => { logger.filter(None, LevelFilter::Debug); },
        }

        // initialize our logger
        logger.init();
    }
}

fn main() {
    let _con = database::Database::open("figure_tracker.db");

    let app: FigureTracker = FigureTracker::parse();
    app.initialize_logger();

    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    match app.subcmd {
        AddSubCommand::Add(t) => {
            match t.subcmd {
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