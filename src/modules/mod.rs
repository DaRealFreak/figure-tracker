use std::convert::TryFrom;
use std::error::Error;

use crate::database::items::Item;
use crate::database::prices::Price;
use crate::modules::amiami::AmiAmi;
use crate::modules::myfigurecollection::MyFigureCollection;

pub(crate) mod amiami;
pub(crate) mod myfigurecollection;

/// Prices is a simple struct for prices including an option for new and used conditions
struct Prices {
    used: Option<Price>,
    new: Option<Price>,
}

/// Module contains the public functionality you can use from the module pool
trait Module {
    fn get_lowest_prices(&self, item: Item) -> Result<Prices, Box<dyn Error>>;
}

/// BaseModule contains all the functionality required from the implemented modules
trait BaseModule {
    fn get_module_key(&self) -> String;
    fn get_lowest_prices(&self, item: Item) -> Result<Prices, Box<dyn Error>>;
    fn matches_url(&self, url: &str) -> bool;
}

/// InfoModule are special modules with the additional functionality to update the item details
trait InfoModule {
    fn get_module_key(&self) -> String;
    fn update_figure_details(&self, item: &mut Item) -> Result<(), Box<dyn Error>>;
}

impl<T> Module for T
where
    T: BaseModule,
{
    /// retrieve the lowest prices for new and used items
    fn get_lowest_prices(&self, item: Item) -> Result<Prices, Box<dyn Error>> {
        debug!("checking prices from module: {:?}", self.get_module_key());
        self.get_lowest_prices(item)
    }
}

/// ModulePool is the main pool for all implemented modules
pub(crate) struct ModulePool {
    modules: Vec<Box<dyn Module>>,
    info_modules: Vec<Box<dyn InfoModule>>,
}

/// implementation of the module pool
impl ModulePool {
    /// returns the module pool with all the implemented modules
    pub fn new() -> Self {
        ModulePool {
            modules: vec![
                Box::from(MyFigureCollection::new()),
                Box::from(AmiAmi::new()),
            ],
            info_modules: vec![
                Box::from(MyFigureCollection::new()),
                Box::from(AmiAmi::new()),
            ],
        }
    }

    /// checks all modules for the prices of the passed item
    pub fn check_item(&self, item: Item) -> Vec<Price> {
        let mut collected_prices: Vec<Price> = vec![];

        self.modules
            .iter()
            .for_each(|module| match module.get_lowest_prices(item.to_owned()) {
                Ok(prices) => {
                    if prices.new.is_some() {
                        collected_prices.push(prices.new.unwrap());
                    }
                    if prices.used.is_some() {
                        collected_prices.push(prices.used.unwrap());
                    }
                }
                Err(err) => warn!("error checking for prices (err: \"{}\")", err.description()),
            });

        collected_prices
    }

    /// iterates through the info modules and tries to update the item information
    pub fn update_info(&self, item: &mut Item) -> Result<(), Box<dyn Error>> {
        for module in self.info_modules.iter() {
            match module.update_figure_details(item) {
                Ok(_) => {
                    info!(
                        "updated figure information from module \"{}\" (title: \"{}\", term: \"{}\")",
                        module.get_module_key(),
                        item.description, item.term_en
                    );
                    return Ok(());
                }
                Err(err) => warn!(
                    "unable to update figure information from module \"{}\" (err: \"{}\")",
                    module.get_module_key(),
                    err.description()
                ),
            }
        }

        Err(
            Box::try_from("unable to retrieve figure information from any module implementation")
                .unwrap(),
        )
    }
}
