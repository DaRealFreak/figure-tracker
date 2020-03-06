use clap::Clap;

use crate::conditions::Conditions;

/// This application tracks wished items on multiple seller/auction sites
/// and notifies the user about new sales/price drops and price averages
#[derive(Clap, Debug)]
#[clap(author, about, version)]
pub(crate) struct FigureTrackerOptions {
    /// Use a custom configuration file.
    #[clap(short = "c", long = "config", default_value = "tracker.yaml")]
    pub(crate) config: String,
    /// A level of verbosity, and can be used multiple times
    #[clap(short = "v", long = "verbose", parse(from_occurrences))]
    pub(crate) verbose: i32,
    #[clap(subcommand)]
    pub(crate) subcmd: SubCommand,
}

#[derive(Clap, Debug)]
pub(crate) enum SubCommand {
    #[clap(name = "add")]
    Add(Add),
    #[clap(name = "update")]
    Update(Update),
}

/// Add an item or notification condition to the database
#[derive(Clap, Debug)]
pub(crate) struct Add {
    #[clap(subcommand)]
    pub(crate) subcmd: AddSubCommand,
}

#[derive(Clap, Debug)]
pub(crate) enum AddSubCommand {
    #[clap(name = "item")]
    Item(AddItem),
    #[clap(name = "notification")]
    Notification(AddNotification),
}

/// Update prices or items in to the database
#[derive(Clap, Debug)]
pub(crate) struct Update {
    #[clap(subcommand)]
    pub(crate) subcmd: UpdateSubCommand,
}

#[derive(Clap, Debug)]
pub(crate) enum UpdateSubCommand {
    #[clap(name = "item")]
    Item(UpdateItem),
    #[clap(name = "prices")]
    Prices(UpdatePrices),
}

/// Add an item to the database
#[derive(Clap, Debug)]
pub(crate) struct AddItem {
    /// JAN numbers of the items to add to the tracked items
    pub(crate) input: Vec<i64>,
}

/// Add a notification condition to the database
#[derive(Clap, Debug)]
pub(crate) struct AddNotification {
    #[clap(
        short = "c",
        long = "condition",
        required = true,
        long_help = r"condition when to notify you about a newly detected price
possible conditions are:
 - below_price - notifies you when the converted price is below <value>
 - below_price_taxed - notification when the converted price including the taxes is below <value>
 - below_price_full - notification when the converted price including taxes and shipping is below <value>
 - lowest_price - notification when the price reached it's lowest point since tracking the prices
 - price_drop - notification is a price is <value> percent lower than the previously detected price
 "
    )]
    pub(crate) condition: Conditions,
    /// value of the notification condition
    #[clap(short = "v", long = "value", required = true)]
    pub(crate) value: f64,
}

/// Update an item manually in the database
#[derive(Clap, Debug)]
pub(crate) struct UpdateItem {}

/// Update prices from all registered modules
#[derive(Clap, Debug)]
pub(crate) struct UpdatePrices {}
