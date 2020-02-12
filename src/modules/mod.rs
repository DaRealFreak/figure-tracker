use std::error::Error;
use std::process;

use crate::modules::myfigurecollection::MyFigureCollection;

mod myfigurecollection;

/// Price is a simple struct for prices including an option for new and used conditions
struct Price {
    used: f64,
    new: f64,
}

/// Module contains the shared functionality between the modules
trait Module {
    fn get_lowest_price(&self) -> Result<Price, Box<dyn Error>>;
}

/// BaseModule contains all the functionality required from the implemented modules
trait BaseModule {
    fn get_module_key(&self) -> &str;
    fn matches_url(&self, url: &str) -> bool;
}

impl<T> Module for T
    where
        T: BaseModule,
{
    fn get_lowest_price(&self) -> Result<Price, Box<dyn Error>> {
        debug!("checking price from module: {:?}", self.get_module_key());

        Ok(Price {
            new: 16.0,
            used: 15.0,
        })
    }
}

/// small test function for the Module implementation
#[test]
pub fn test() {
    let mfc = MyFigureCollection {};

    match mfc.get_lowest_price() {
        Ok(price_info) => info!(
            "price info: new -> {:?}, used -> {:?}",
            price_info.new, price_info.used
        ),
        Err(err) => {
            warn!(
                "unable to retrieve lowest price from module {:?} (err: {})",
                mfc.get_module_key(),
                err.description()
            );
            process::exit(1)
        }
    }
}
