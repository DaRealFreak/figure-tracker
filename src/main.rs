use clap::Clap;
use rusqlite::Error;

mod database;

#[derive(Clap)]
#[clap(version = "1.0", author = "DaRealFreak")]
struct Opts {
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
    /// Print debug info
    #[clap(short = "d")]
    debug: bool,
    /// Some input. Because this isn't an Option<T> it's required to be used
    input: Option<String>,
}

fn main() {
    let _con = database::Database::open("figure_tracker.db");

    let opts: Opts = Opts::parse();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    println!("Value for config: {}", opts.config);

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    match opts.verbose {
        0 => println!("No verbose info"),
        1 => println!("Some verbose info"),
        2 => println!("Tons of verbose info"),
        3 | _ => println!("Don't be crazy"),
    }

    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    match opts.subcmd {
        AddSubCommand::Add(t) => {
            println!("add command executed")
        }
        _ => {}
    }
}