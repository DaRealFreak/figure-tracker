use std::error::Error;

pub(crate) mod myfigurecollection;

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
