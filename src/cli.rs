use clap::Clap;

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

/// Add an account or item to the database
#[derive(Clap, Debug)]
pub(crate) struct Add {
    #[clap(subcommand)]
    pub(crate) subcmd: AddSubCommand,
}

#[derive(Clap, Debug)]
pub(crate) enum AddSubCommand {
    #[clap(name = "item")]
    Item(AddItem),
    #[clap(name = "account")]
    Account(AddAccount),
}

/// Update prices, items or accounts in to the database
#[derive(Clap, Debug)]
pub(crate) struct Update {
    #[clap(subcommand)]
    pub(crate) subcmd: UpdateSubCommand,
}

#[derive(Clap, Debug)]
pub(crate) enum UpdateSubCommand {
    #[clap(name = "item")]
    Item(UpdateItem),
    #[clap(name = "account")]
    Account(UpdateAccount),
    #[clap(name = "prices")]
    Prices(UpdatePrices),
}

/// Add an item to the database
#[derive(Clap, Debug)]
pub(crate) struct AddItem {
    /// JAN numbers of the items to add to the tracked items
    pub(crate) input: Vec<i64>,
}

/// Add an account to the database
#[derive(Clap, Debug)]
pub(crate) struct AddAccount {
    /// username of the account to add
    #[clap(short = "u", long = "username")]
    pub(crate) username: String,
    /// password of the account to add
    #[clap(short = "p", long = "password")]
    pub(crate) password: String,
    /// url of the related site of the account to add
    #[clap(short = "U", long = "url")]
    pub(crate) url: String,
}

/// Update an item manually in the database
#[derive(Clap, Debug)]
pub(crate) struct UpdateItem {}

/// Update an account manually in the database
#[derive(Clap, Debug)]
pub(crate) struct UpdateAccount {}

/// Update prices from all registered modules
#[derive(Clap, Debug)]
pub(crate) struct UpdatePrices {}
