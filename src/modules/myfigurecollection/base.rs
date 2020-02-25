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

    fn get_lowest_prices(&self, item: &Item) -> Result<Prices, Box<dyn Error>> {
        let _search_url = format!(
            "https://myfigurecollection.net/classified.php?type=0&itemId={}",
            self.get_figure_id(&item)?
        );
        // ToDo: retrieving HTML and parsing results

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

#[test]
pub fn test_get_lowest_prices() {
    let item = &mut Item {
        id: 0,
        jan: 4_580_416_940_283,
        description: "".to_string(),
        term_en: "".to_string(),
        term_jp: "".to_string(),
        disabled: false,
    };

    let mfc = MyFigureCollection {
        client: reqwest::blocking::Client::builder().build().unwrap(),
    };

    assert!(mfc.get_lowest_prices(item).is_ok());
    if let Err(err) = mfc.get_lowest_prices(item) {
        println!("{}", err)
    }
}
