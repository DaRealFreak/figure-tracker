use std::error::Error;

use kuchiki::traits::TendrilSink;
use regex::Regex;

use crate::database::items::Item;
use crate::database::prices::Price;
use crate::modules::amazon::AmazonCoJp;
use crate::modules::{BaseModule, Prices};

struct Base<'a> {
    pub(crate) client: &'a reqwest::blocking::Client,
}

impl<'a> Base<'a> {
    fn get_lowest_new_price(&self, asin: &'a str) -> Result<Option<Price>, Box<dyn Error>> {
        let search_url = format!(
            "https://neokyo.com/amazon-marketplace-listing\
            ?provider=amazonJapan&asin={}&item_title=&new=true",
            asin
        );
        println!("checking url for new prices: {}", search_url);
        Ok(None)
    }

    fn get_lowest_used_price(&self, asin: &'a str) -> Result<Option<Price>, Box<dyn Error>> {
        let search_url = format!(
            "https://neokyo.com/amazon-marketplace-listing\
            ?provider=amazonJapan\
            &asin={}&item_title=&used=true&as_new=true&very_good=true&good=true&acceptable=true",
            asin
        );
        println!("checking url for used prices: {}", search_url);
        Ok(None)
    }
}

impl BaseModule for AmazonCoJp {
    /// retrieve the module key
    fn get_module_key(&self) -> String {
        AmazonCoJp::get_module_key()
    }

    /// retrieve the lowest price for new and used condition
    fn get_lowest_prices(&self, item: &Item) -> Result<Prices, Box<dyn Error>> {
        let search_url = format!(
            "https://neokyo.com/search-results?keyword={}&provider=amazonJapan&spid=",
            item.jan
        );

        let mut prices = Prices {
            used: None,
            new: None,
        };

        let base = Base {
            client: &self.client,
        };
        let res = self.client.get(search_url.as_str()).send()?;
        let doc = kuchiki::parse_html().one(res.text()?.as_str());

        for css_match in doc
            .select("div.product-card a.nippon-cta.btn[href*='neokyo.com/product/']")
            .unwrap()
        {
            if let Some(element) = css_match.as_node().as_element() {
                if let Some(detail_link) = element.attributes.borrow().get("href") {
                    let asin_regex = Regex::new(r"/(?P<asin>[^/]*)\?")?;

                    if asin_regex.is_match(detail_link) {
                        if let Some(asin) = asin_regex.captures(detail_link).unwrap().name("asin") {
                            prices.new = base.get_lowest_new_price(asin.as_str())?;
                            prices.used = base.get_lowest_used_price(asin.as_str())?;
                        }
                    }
                }
            }
        }

        Ok(prices)
    }
}

#[test]
pub fn test_get_lowest_prices() {
    let item = &mut Item {
        id: 0,
        jan: 4_934_054_783_441,
        description: "".to_string(),
        term_en: "".to_string(),
        term_jp: "".to_string(),
        disabled: false,
    };

    let amazon = AmazonCoJp {
        client: reqwest::blocking::Client::builder().build().unwrap(),
    };

    if let Err(err) = amazon.get_lowest_prices(item) {
        println!("{}", err)
    }
}
