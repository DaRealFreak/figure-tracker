use std::borrow::Borrow;
use std::error::Error;

use crate::database::items::Item;
use crate::modules::myfigurecollection::MyFigureCollection;

pub(crate) mod myfigurecollection;

/// Price is a simple struct for prices including an option for new and used conditions
struct Price {
    used: f64,
    new: f64,
}

/// Module contains the public functionality you can use from the module pool
trait Module {
    fn get_module_key(&self) -> &str;
    fn get_lowest_price(&self, item: Item) -> Result<Price, Box<dyn Error>>;
}

/// BaseModule contains all the functionality required from the implemented modules
trait BaseModule {
    fn new() -> Self;
    fn get_module_key(&self) -> &str;
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
    fn get_lowest_price(&self, item: Item) -> Result<Price, Box<dyn Error>> {
        debug!("checking price from module: {:?}", self.get_module_key());

        Ok(Price {
            new: 16.0,
            used: 15.0,
        })
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
    pub fn check_item(&self, item: Item) {
        let modules: &[Box<dyn Module>] = self.modules.borrow();
        modules
            .into_iter()
            .for_each(|module| match module.get_lowest_price(item.to_owned()) {
                Ok(price) => info!(
                    "detected price for \"{}\" from module: \"{}\": new \"{:?}\", used \"{:?}\"",
                    item.term,
                    module.get_module_key(),
                    price.new,
                    price.used
                ),
                Err(err) => warn!("error checking for prices (err: \"{}\")", err.description()),
            });
    }
}
