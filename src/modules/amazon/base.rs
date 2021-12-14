use std::error::Error;

use kuchiki::traits::TendrilSink;
use regex::Regex;

use crate::currency::guesser::CurrencyGuesser;
use crate::currency::SupportedCurrency;
use crate::database::items::{Item, ItemConditions};
use crate::database::prices::Price;
use crate::modules::amazon::AmazonCoJp;
use crate::modules::{BaseModule, Prices};

struct Base<'a> {
    pub(crate) client: &'a reqwest::blocking::Client,
}

impl<'a> Base<'a> {
    // don't ask me how but reqwest fools the IntelliJ plugin quite good
    //noinspection RsUnresolvedReference
    fn get_lowest_price_from_url(
        &self,
        item: Item,
        url: String,
        condition: ItemConditions,
    ) -> Result<Option<Price>, Box<dyn Error>> {
        let mut prices = vec![];

        let res = self.client.get(url.as_str()).send()?;
        let doc = kuchiki::parse_html().one(res.text()?.as_str());
        for sale in doc
            .select("section.main-section-product-details div.col-lg-10 div.container")
            .unwrap()
        {
            let sale_node = sale.as_node();
            let price_text = sale_node
                .select_first("strong.text-green.price")
                .unwrap()
                .text_contents();

            prices.push(Price::new(
                item.clone(),
                CurrencyGuesser::get_currency_value(price_text)?,
                SupportedCurrency::JPY,
                url.clone(),
                AmazonCoJp::get_module_key(),
                condition,
            ));
        }

        let mut lowest_price: Option<Price> = None;
        for price in prices {
            if lowest_price.is_none() || price.price < lowest_price.clone().unwrap().price {
                lowest_price = Some(price)
            }
        }

        Ok(lowest_price)
    }

    fn get_lowest_new_price(
        &self,
        item: Item,
        asin: &'a str,
    ) -> Result<Option<Price>, Box<dyn Error>> {
        let search_url = format!(
            "https://neokyo.com/en/amazon-marketplace-listing\
            ?provider=amazonJapan&asin={}&item_title=&new=true",
            asin
        );

        self.get_lowest_price_from_url(item, search_url, ItemConditions::New)
    }

    fn get_lowest_used_price(
        &self,
        item: Item,
        asin: &'a str,
    ) -> Result<Option<Price>, Box<dyn Error>> {
        let search_url = format!(
            "https://neokyo.com/en/amazon-marketplace-listing\
             ?provider=amazonJapan\
             &asin={}&item_title=&used=true&as_new=true&very_good=true&good=true&acceptable=true",
            asin
        );

        self.get_lowest_price_from_url(item, search_url, ItemConditions::Used)
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
            "https://neokyo.com/en/search-results?keyword={}&provider=amazonJapan&spid=",
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

        if let Ok(css_match) =
            doc.select_first("div.product-card a.nippon-cta.btn[href*='neokyo.com/en/product/']")
        {
            if let Some(element) = css_match.as_node().as_element() {
                if let Some(detail_link) = element.attributes.borrow().get("href") {
                    let asin_regex = Regex::new(r"/(?P<asin>[^/]*)\?")?;

                    if asin_regex.is_match(detail_link) {
                        if let Some(asin) = asin_regex.captures(detail_link).unwrap().name("asin") {
                            prices.new = base.get_lowest_new_price(item.clone(), asin.as_str())?;
                            prices.used =
                                base.get_lowest_used_price(item.clone(), asin.as_str())?;
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
        image: "".to_string(),
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
