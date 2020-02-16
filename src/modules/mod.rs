use std::borrow::Borrow;
use std::error::Error;

use crate::database::items::Item;
use crate::database::prices::Price;
use crate::modules::myfigurecollection::MyFigureCollection;

pub(crate) mod myfigurecollection;

/// Prices is a simple struct for prices including an option for new and used conditions
struct Prices {
    used: Option<Price>,
    new: Option<Price>,
}

/// Module contains the public functionality you can use from the module pool
trait Module {
    fn get_module_key(&self) -> &str;
    fn get_lowest_prices(&self, item: Item) -> Result<Prices, Box<dyn Error>>;
}

/// BaseModule contains all the functionality required from the implemented modules
trait BaseModule {
    fn new() -> Self;
    fn get_module_key(&self) -> &str;
    fn get_lowest_prices(&self, item: Item) -> Result<Prices, Box<dyn Error>>;
    fn matches_url(&self, url: &str) -> bool;
}

impl<T> Module for T
where
    T: BaseModule,
{
    /// return the module key from the current module
    fn get_module_key(&self) -> &str {
        self.get_module_key()
    }

    /// retrieve the lowest prices for new and used items
    fn get_lowest_prices(&self, item: Item) -> Result<Prices, Box<dyn Error>> {
        debug!("checking prices from module: {:?}", self.get_module_key());
        self.get_lowest_prices(item.clone())
    }
}

/// ModulePool is the main pool for all implemented modules
pub(crate) struct ModulePool {
    modules: Vec<Box<dyn Module>>,
}

/// implementation of the module pool
impl ModulePool {
    /// returns the module pool with all the implemented modules
    pub fn new() -> Self {
        ModulePool {
            modules: vec![Box::from(MyFigureCollection::new())],
        }
    }

    /// checks all modules for the prices of the passed item
    pub fn check_item(&self, item: Item) -> Vec<Price> {
        let mut collected_prices: Vec<Price> = vec![];

        let modules: &[Box<dyn Module>] = self.modules.borrow();
        modules
            .into_iter()
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
}
