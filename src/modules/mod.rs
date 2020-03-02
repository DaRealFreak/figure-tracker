use std::convert::TryFrom;
use std::error::Error;
use std::sync::{Arc, Barrier, Mutex};

use threadpool::ThreadPool;

use crate::configuration::Configuration;
use crate::currency::conversion::CurrencyConversion;
use crate::currency::guesser::CurrencyGuesser;
use crate::database::items::Item;
use crate::database::prices::Price;
use crate::modules::amiami::AmiAmi;
use crate::modules::myfigurecollection::MyFigureCollection;
use crate::modules::solarisjapan::SolarisJapan;

pub(crate) mod amiami;
pub(crate) mod myfigurecollection;
pub(crate) mod solarisjapan;

/// Prices is a simple struct for prices including an option for new and used conditions
struct Prices {
    used: Option<Price>,
    new: Option<Price>,
}

/// Module contains the public functionality you can use from the module pool
trait Module {
    fn get_lowest_prices(&self, item: &Item) -> Result<Prices, Box<dyn Error>>;
}

/// BaseModule contains all the functionality required from the implemented modules
trait BaseModule: BaseModuleClone {
    fn get_module_key(&self) -> String;
    fn get_lowest_prices(&self, item: &Item) -> Result<Prices, Box<dyn Error>>;
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
    fn get_lowest_prices(&self, item: &Item) -> Result<Prices, Box<dyn Error>> {
        debug!("checking prices from module: {:?}", self.get_module_key());
        self.get_lowest_prices(item)
    }
}

/// BaseModuleClone trait to assert the clone_box function
trait BaseModuleClone {
    fn clone_box(&self) -> Box<dyn BaseModule + Send + Sync>;
}

/// use the box clone to implement the Clone trait for thread shareable BaseModule implementations
impl<T> BaseModuleClone for T
where
    T: 'static + BaseModule + Clone + Send + Sync,
{
    fn clone_box(&self) -> Box<dyn BaseModule + Send + Sync> {
        Box::new(self.clone())
    }
}

/// clone implementation for our thread safe BaseModule implementation
impl Clone for Box<dyn BaseModule + Send + Sync> {
    fn clone(&self) -> Box<dyn BaseModule + Send + Sync> {
        self.clone_box()
    }
}

/// ModulePool is the main pool for all implemented modules
pub(crate) struct ModulePool {
    modules: Vec<Box<dyn BaseModule + Send + Sync>>,
    info_modules: Vec<Box<dyn InfoModule + Send + Sync>>,
    conversion: CurrencyConversion,
}

/// implementation of the module pool
impl ModulePool {
    /// returns the module pool with all the implemented modules
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let conversion = CurrencyConversion::new()?;
        Ok(ModulePool {
            modules: vec![
                Box::from(MyFigureCollection::new(conversion.clone())?),
                Box::from(AmiAmi::new()?),
                Box::from(SolarisJapan::new()?),
            ],
            info_modules: vec![
                Box::from(MyFigureCollection::new(conversion.clone())?),
                Box::from(AmiAmi::new()?),
            ],
            conversion,
        })
    }

    /// checks all modules for the prices of the passed item
    pub fn check_item(&self, item: Item) -> Vec<Price> {
        let collected_prices: Arc<Mutex<Vec<Price>>> = Arc::new(Mutex::new(vec![]));

        let pool = ThreadPool::new(self.modules.len());
        let barrier = Arc::new(Barrier::new(self.modules.len() + 1));

        for i in 0..self.modules.len() {
            let barrier = barrier.clone();
            let item = item.clone();
            let module = self.modules[i].clone();
            let collected_prices = collected_prices.clone();
            let conversion = self.conversion.clone();

            pool.execute(move || {
                let mut collected_prices = collected_prices.lock().unwrap();
                match module.get_lowest_prices(&item) {
                    Ok(prices) => {
                        for price_option in vec![prices.new, prices.used] {
                            if let Some(mut price) = price_option {
                                if let Some(currency) = CurrencyGuesser::new()
                                    .guess_currency(price.currency.as_str().to_string())
                                {
                                    price.converted_price = conversion.convert_price_to(
                                        price.price,
                                        currency,
                                        Configuration::get_used_currency(),
                                    );
                                    price.converted_currency =
                                        Configuration::get_used_currency().to_string();
                                }
                                collected_prices.push(price);
                            }
                        }
                    }
                    Err(err) => warn!(
                        "[{}] - error checking for prices for {:?} (err: {:?})",
                        module.get_module_key(),
                        item.description,
                        err
                    ),
                }

                // release the collected prices again
                drop(collected_prices);

                // wait for the other threads
                barrier.wait();
            });
        }

        // wait for all threads to finish their work
        barrier.wait();

        #[allow(clippy::redundant_clone)]
        collected_prices.clone().lock().unwrap().to_vec()
    }

    /// iterates through the info modules and tries to update the item information
    pub fn update_info(&self, item: &mut Item) -> Result<(), Box<dyn Error>> {
        for module in self.info_modules.iter() {
            match module.update_figure_details(item) {
                Ok(_) => {
                    info!(
                        "updated figure information from module {:?} (title: {:?}, term_en: {:?}, term_jp: {:?})",
                        module.get_module_key(),
                        item.description, item.term_en, item.term_jp
                    );
                    return Ok(());
                }
                Err(err) => warn!(
                    "unable to update figure information from module {:?} (err: {:?})",
                    module.get_module_key(),
                    err
                ),
            }
        }

        Err(
            Box::try_from("unable to retrieve figure information from any module implementation")
                .unwrap(),
        )
    }
}
