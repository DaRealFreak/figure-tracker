use std::error::Error;

use chrono::Utc;

use crate::database::items::{Item, ItemConditions};
use crate::database::prices::Price;
use crate::modules::myfigurecollection::MyFigureCollection;
use crate::modules::{BaseModule, Prices};

impl BaseModule for MyFigureCollection {
    fn get_module_key(&self) -> String {
        MyFigureCollection::get_module_key()
    }

    fn get_lowest_prices(&self, item: Item) -> Result<Prices, Box<dyn Error>> {
        Ok(Prices {
            new: Option::from(Price {
                id: None,
                price: 10.02,
                url: "".to_string(),
                module: MyFigureCollection::get_module_key(),
                currency: "¥".to_string(),
                condition: ItemConditions::New,
                timestamp: Utc::now(),
            }),
            used: None,
        })
    }

    fn matches_url(&self, _url: &str) -> bool {
        unimplemented!("not implemented yet")
    }
}
