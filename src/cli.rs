use clap::Parser;

use crate::conditions::ConditionType;
use crate::database::items::ItemConditions;

/// This application tracks wished items on multiple seller/auction sites
/// and notifies the user about new sales/price drops and price averages
#[derive(Parser, Debug)]
#[clap(author, about, version)]
pub(crate) struct FigureTrackerOptions {
    /// Use a custom configuration file.
    #[clap(short = 'c', long = "config", default_value = "tracker.yaml")]
    pub(crate) config: String,
    /// A level of verbosity, and can be used multiple times
    #[clap(short = 'v', long = "verbose", parse(from_occurrences))]
    pub(crate) verbose: i32,
    #[clap(subcommand)]
    pub(crate) subcmd: SubCommand,
}

#[derive(Parser, Debug)]
pub(crate) enum SubCommand {
    #[clap(name = "add")]
    Add(Add),
    #[clap(name = "update")]
    Update(Update),
}

/// Add an item or notification condition to the database
#[derive(Parser, Debug)]
pub(crate) struct Add {
    #[clap(subcommand)]
    pub(crate) subcmd: AddSubCommand,
}

#[derive(Parser, Debug)]
pub(crate) enum AddSubCommand {
    #[clap(name = "item")]
    Item(AddItem),
    #[clap(name = "notification")]
    Notification(AddNotification),
}

/// Update prices or items in to the database
#[derive(Parser, Debug)]
pub(crate) struct Update {
    #[clap(subcommand)]
    pub(crate) subcmd: UpdateSubCommand,
}

#[derive(Parser, Debug)]
pub(crate) enum UpdateSubCommand {
    #[clap(name = "item")]
    Item(UpdateItem),
    #[clap(name = "prices")]
    Prices(UpdatePrices),
}

/// Add an item to the database
#[derive(Parser, Debug)]
pub(crate) struct AddItem {
    /// JAN/EAN numbers of the items to add to the tracked items
    #[clap(required = true, min_values = 1)]
    pub(crate) input: Vec<i64>,
}

/// Add a notification condition to the database
#[derive(Parser, Debug)]
pub(crate) struct AddNotification {
    #[clap(
        short = 't',
        long = "type",
        required = true,
        long_help = r"condition type when to notify you about a newly detected price
possible types are:
 - below_price - notifies you when the converted price is below <value>
 - below_price_taxed - notification when the converted price including the taxes is below <value>
 - below_price_full - notification when the converted price including taxes and shipping is below <value>
 - lowest_price - notification when the price is <value> below it's lowest point since tracking the prices
 - price_drop - notification is a price is <value> percent lower than the previously detected price"
    )]
    pub(crate) condition_type: ConditionType,
    #[clap(
        short = 'c',
        long = "condition",
        long_help = r"option to limit notifications for a specific item condition
possible conditions are:
 - new - figure is still unopened in the box
 - used - figure is used and box got opened already"
    )]
    pub(crate) condition: Option<ItemConditions>,
    /// value of the notification condition
    #[clap(short = 'v', long = "value", required = true)]
    pub(crate) value: f64,
    /// JAN/EAN numbers of the items to add the notification condition for
    #[clap(required = true, min_values = 1)]
    pub(crate) items: Vec<i64>,
}

/// Update an item manually in the database
#[derive(Parser, Debug)]
pub(crate) struct UpdateItem {}

/// Update prices from all registered modules
#[derive(Parser, Debug)]
pub(crate) struct UpdatePrices {}
