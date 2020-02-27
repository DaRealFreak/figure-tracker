use std::error::Error;

use chrono::Utc;
use scraper::{ElementRef, Html, Selector};

use crate::currency::{CurrencyGuesser, SupportedCurrency};
use crate::database::items::{Item, ItemConditions};
use crate::database::prices::Price;
use crate::modules::myfigurecollection::MyFigureCollection;
use crate::modules::{BaseModule, Prices};

struct Base<'a> {
    pub(crate) inner: &'a MyFigureCollection,
}

impl<'a> Base<'a> {
    fn get_sales(doc: &Html) -> Vec<ElementRef> {
        let element_selector = Selector::parse("ul.listing li.listing-item").unwrap();
        doc.select(&element_selector).collect()
    }

    fn get_sale_price(element: &ElementRef<'a>) -> String {
        let price_selector = Selector::parse("span.classified-price-value").unwrap();
        element
            .select(&price_selector)
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap()
            .to_string()
    }

    fn get_sale_currency(element: &ElementRef<'a>) -> String {
        let currency_selector = Selector::parse("span.classified-price-currency").unwrap();
        element
            .select(&currency_selector)
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap()
            .to_string()
    }
}

impl BaseModule for MyFigureCollection {
    fn get_module_key(&self) -> String {
        MyFigureCollection::get_module_key()
    }

    fn get_lowest_prices(&self, item: &Item) -> Result<Prices, Box<dyn Error>> {
        let search_url = format!(
            "https://myfigurecollection.net/classified.php?type=0&itemId={}",
            self.get_figure_id(&item)?
        );
        let res = self.client.get(search_url.as_str()).send()?;
        let doc = Html::parse_document(res.text()?.as_str());

        for element in Base::get_sales(&doc) {
            let currency = Base::get_sale_currency(&element);
            let price = Base::get_sale_price(&element);

            if let Some(currency) = CurrencyGuesser::new().guess_currency(currency) {
                if let Ok(converted_price) = CurrencyGuesser::get_currency_value(price.clone()) {
                    println!(
                        "{} {} -> {:.2} {}",
                        price,
                        currency,
                        self.conversion.convert_price_to(
                            converted_price,
                            currency.clone(),
                            SupportedCurrency::EUR
                        ),
                        SupportedCurrency::EUR
                    );
                }
            }
        }

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
    use crate::currency::CurrencyConversion;
    use std::collections::BTreeMap;

    let item = &mut Item {
        id: 0,
        jan: 4_580_416_940_283,
        description: "".to_string(),
        term_en: "".to_string(),
        term_jp: "".to_string(),
        disabled: false,
    };

    let mfc = MyFigureCollection {
        conversion: CurrencyConversion {
            exchange_rates: BTreeMap::new(),
        },
        client: reqwest::blocking::Client::builder().build().unwrap(),
    };

    assert!(mfc.get_lowest_prices(item).is_ok());
    if let Err(err) = mfc.get_lowest_prices(item) {
        println!("{}", err)
    }
}
