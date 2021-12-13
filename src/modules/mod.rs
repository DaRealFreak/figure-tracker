use core::fmt;
use std::error::Error;
use std::sync::{Arc, Barrier, Mutex};

use std::fmt::Formatter;
use threadpool::ThreadPool;

use crate::configuration::Configuration;
use crate::currency::conversion::CurrencyConversion;
use crate::currency::guesser::CurrencyGuesser;
use crate::database::items::Item;
use crate::database::prices::Price;
use crate::modules::amazon::AmazonCoJp;
use crate::modules::amiami::AmiAmi;
use crate::modules::myfigurecollection::MyFigureCollection;
use crate::modules::rakuten::Rakuten;
use crate::modules::solarisjapan::SolarisJapan;

pub(crate) mod amazon;
pub(crate) mod amiami;
pub(crate) mod myfigurecollection;
pub(crate) mod rakuten;
pub(crate) mod solarisjapan;

/// Prices is a simple struct for prices including an option for new and used conditions
struct Prices {
    used: Option<Price>,
    new: Option<Price>,
}

/// generic error type to be returned from modules for if the figure couldn't be found
#[derive(Debug)]
struct NotFoundError {}

/// implement display trait for NotFoundError
impl std::fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "searched figure could not be found")
    }
}

/// mark as error implementation
impl Error for NotFoundError {}

/// custom error if no information could be retrieved from any info module
#[derive(Debug)]
struct NoInfoFromModulesError {}

/// implement display trait for NoInfoFromModulesError
impl std::fmt::Display for NoInfoFromModulesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unable to retrieve figure information from any module implementation"
        )
    }
}

/// mark as error implementation
impl Error for NoInfoFromModulesError {}

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
                Box::from(AmazonCoJp::new()?),
                Box::from(Rakuten::new()?),
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

        let used_currency = Configuration::get_used_currency();
        let pool = ThreadPool::new(self.modules.len());
        let barrier = Arc::new(Barrier::new(self.modules.len() + 1));

        for i in 0..self.modules.len() {
            let barrier = barrier.clone();
            let item = item.clone();
            let module = self.modules[i].clone();
            let collected_prices = collected_prices.clone();
            let conversion = self.conversion.clone();
            let used_currency = used_currency.clone();

            pool.execute(move || {
                match module.get_lowest_prices(&item) {
                    Ok(prices) => {
                        for price_option in vec![prices.new, prices.used] {
                            if let Some(mut price) = price_option {
                                if let Some(currency) = CurrencyGuesser::new()
                                    .guess_currency(price.currency.as_str().to_string())
                                {
                                    price.converted_price = conversion.convert_price_to(
                                        price.price,
                                        currency.clone(),
                                        used_currency.clone(),
                                    );
                                    price.converted_currency = used_currency.to_string();
                                    price.shipping = Configuration::get_shipping(currency.clone());
                                    price.taxes =
                                        Configuration::get_used_tax_rate(currency.clone());
                                }
                                info!(
                                    "[{}] - detected price for {:?}: price: {:.2} {} (without shipping/taxes: {:.2} {} / {:.2} {}), condition: {:?}",
                                    price.module, item.description, price.get_converted_total(), price.converted_currency, price.price, price.currency, price.converted_price, price.converted_currency, price.condition,
                                );

                                // push our result into the collected prices and release them again
                                let mut collected_prices = collected_prices.lock().unwrap();
                                collected_prices.push(price);
                                drop(collected_prices);
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

        Err(Box::from(NoInfoFromModulesError {}))
    }
}
