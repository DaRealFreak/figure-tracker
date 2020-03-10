use std::error::Error;

use kuchiki::traits::TendrilSink;

use crate::currency::guesser::CurrencyGuesser;
use crate::currency::SupportedCurrency;
use crate::database::items::{Item, ItemConditions};
use crate::database::prices::Price;
use crate::modules::rakuten::Rakuten;
use crate::modules::{BaseModule, Prices};

impl BaseModule for Rakuten {
    /// retrieve the module key
    fn get_module_key(&self) -> String {
        Rakuten::get_module_key()
    }

    /// retrieve the lowest price for new and used condition
    fn get_lowest_prices(&self, item: &Item) -> Result<Prices, Box<dyn Error>> {
        let mut prices = Prices {
            used: None,
            new: None,
        };

        let mut collected_prices = vec![];

        let search_url = format!("https://search.rakuten.co.jp/search/mall/{}/", item.jan);
        let res = self.client.get(search_url.as_str()).send()?;
        let doc = kuchiki::parse_html().one(res.text()?.as_str());

        for css_match in doc
            .select("div.searchresultitems div.searchresultitem")
            .unwrap()
        {
            let sale = css_match.as_node();
            let cond = if let Ok(cond) = sale.select_first("span.dui-tag.-type") {
                match cond.text_contents().as_str() {
                    "中古" => ItemConditions::Used,
                    "新品" => ItemConditions::New,
                    _ => ItemConditions::New,
                }
            } else {
                ItemConditions::New
            };

            let price_text = sale
                .select_first("div.content.price > span")
                .unwrap()
                .text_contents();

            let sale_url = sale
                .select_first("h2 > a[title]")
                .unwrap()
                .attributes
                .borrow()
                .get("href")
                .unwrap()
                .to_string();

            collected_prices.push(Price::new(
                item.clone(),
                CurrencyGuesser::get_currency_value(price_text)?,
                SupportedCurrency::JPY,
                sale_url,
                Rakuten::get_module_key(),
                cond,
            ));
        }

        for price in collected_prices {
            match price.condition {
                ItemConditions::New => {
                    if prices.new.clone().is_none()
                        || price.price < prices.new.clone().unwrap().price
                    {
                        prices.new = Some(price)
                    }
                }
                ItemConditions::Used => {
                    if prices.used.clone().is_none()
                        || price.price < prices.used.clone().unwrap().price
                    {
                        prices.used = Some(price)
                    }
                }
                _ => {}
            }
        }

        Ok(prices)
    }
}

#[test]
pub fn test_get_lowest_prices() {
    let item = &mut Item {
        id: 0,
        jan: 4_545_784_042_649,
        description: "".to_string(),
        image: "".to_string(),
        term_en: "".to_string(),
        term_jp: "".to_string(),
        disabled: false,
    };

    let rakuten = Rakuten {
        client: reqwest::blocking::Client::builder().build().unwrap(),
    };

    match rakuten.get_lowest_prices(item) {
        Ok(prices) => {
            println!("lowest price new: {}", prices.new.unwrap().price);
            println!("lowest price used: {}", prices.used.unwrap().price);
        }
        Err(err) => println!("{}", err),
    }
}
