use std::borrow::Borrow;
use std::error::Error;

use kuchiki::traits::TendrilSink;

use crate::configuration::Configuration;
use crate::currency::guesser::CurrencyGuesser;
use crate::database::items::{Item, ItemConditions};
use crate::database::prices::Price;
use crate::modules::myfigurecollection::MyFigureCollection;
use crate::modules::{BaseModule, Prices};

struct Base<'a> {
    pub(crate) inner: &'a MyFigureCollection,
}

impl<'a> Base<'a> {
    /// extract sales from url and navigate through possibly multiple pages
    fn get_sales_from_url(
        &self,
        item: Item,
        search_url: String,
    ) -> Result<Vec<Price>, Box<dyn Error>> {
        let mut sales = vec![];
        let used_currency = Configuration::get_used_currency();
        let mut res = self.inner.client.get(search_url.as_str()).send()?;

        loop {
            let doc = kuchiki::parse_html().one(res.text()?.as_str());
            for element in doc.select("ul.listing li.listing-item").unwrap() {
                let element_node = element.as_node();
                let currency = element_node
                    .select_first("span.classified-price-currency")
                    .unwrap()
                    .text_contents();
                let price = element_node
                    .select_first("span.classified-price-value")
                    .unwrap()
                    .text_contents();
                let sale_url = element_node
                    .select_first("a.tbx-tooltip[href*='classified']")
                    .unwrap()
                    .attributes
                    .borrow()
                    .get("href")
                    .unwrap()
                    .to_string();

                if let Some(currency) = CurrencyGuesser::new().guess_currency(currency) {
                    if let Ok(price_value) = CurrencyGuesser::get_currency_value(price.clone()) {
                        let mut price = Price::new(
                            item.clone(),
                            price_value,
                            currency.clone(),
                            sale_url,
                            MyFigureCollection::get_module_key(),
                            ItemConditions::New,
                        );
                        price.converted_price = self.inner.conversion.convert_price_to(
                            price.price,
                            currency.clone(),
                            used_currency.clone(),
                        );
                        price.converted_currency = used_currency.clone().to_string();
                        price.taxes = Configuration::get_used_tax_rate(currency.clone());
                        price.shipping = Configuration::get_shipping(currency.clone());

                        sales.push(price);
                    }
                }
            }

            if let Ok(next_page_url) =
                doc.select_first("nav.listing-count-pages > a.nav-next[href]")
            {
                let next_page_url = next_page_url
                    .as_node()
                    .as_element()
                    .unwrap()
                    .attributes
                    .borrow()
                    .get("href")
                    .unwrap()
                    .to_string();
                res = self.inner.client.get(next_page_url.as_str()).send()?;
            } else {
                break;
            }
        }

        Ok(sales)
    }

    /// retrieve all sales
    fn get_all_sales(&self, item: Item, figure_id: u32) -> Result<Vec<Price>, Box<dyn Error>> {
        let search_url = format!(
            "https://myfigurecollection.net/classified.php?type=0&itemId={}",
            figure_id
        );

        self.get_sales_from_url(item, search_url)
    }

    /// retrieve new sales
    fn get_new_sales(&self, item: Item, figure_id: u32) -> Result<Vec<Price>, Box<dyn Error>> {
        let search_url = format!(
            "https://myfigurecollection.net/classified.php?type=0&itemId={}&isMIB=1",
            figure_id
        );

        self.get_sales_from_url(item, search_url)
    }

    /// retrieve used sales
    /// (MFC doesn't display mint/used difference in sales page, so we have to retrieve the differences ourselves)
    fn get_used_sales(all_sales: &[Price], new_sales: &[Price]) -> Vec<Price> {
        let mut sales = vec![];

        'outer: for sale in all_sales {
            let mut sale = sale.clone();
            // check for identical ad IDs of new sales and continue if found
            for new_sale in new_sales {
                if new_sale.url == sale.url {
                    continue 'outer;
                }
            }
            // else it's used and we append it to our used sales
            sale.condition = ItemConditions::Used;
            sales.push(sale)
        }

        sales
    }
}

impl BaseModule for MyFigureCollection {
    /// retrieve the module key
    fn get_module_key(&self) -> String {
        MyFigureCollection::get_module_key()
    }

    /// retrieve the lowest price for new and used condition
    fn get_lowest_prices(&self, item: &Item) -> Result<Prices, Box<dyn Error>> {
        let figure_id = self.get_figure_id(&item)?;
        let all_sales = Base { inner: self }.get_all_sales(item.clone(), figure_id.clone())?;
        let new_sales = Base { inner: self }.get_new_sales(item.clone(), figure_id)?;

        let used_sales = Base::get_used_sales(&all_sales, &new_sales);

        let mut prices = Prices {
            new: None,
            used: None,
        };

        for new_sale in new_sales {
            if prices.borrow().new.is_none()
                || new_sale.get_converted_total()
                    < prices.borrow().new.as_ref().unwrap().get_converted_total()
            {
                prices.new = Some(new_sale);
            }
        }

        for used_sale in used_sales {
            if prices.borrow().used.is_none()
                || used_sale.get_converted_total()
                    < prices.borrow().used.as_ref().unwrap().get_converted_total()
            {
                prices.used = Some(used_sale);
            }
        }

        Ok(prices)
    }
}

#[test]
pub fn test_get_lowest_prices() {
    use crate::currency::conversion::CurrencyConversion;
    use std::collections::BTreeMap;

    let item = &mut Item {
        id: 0,
        jan: 4_580_416_940_283,
        description: "".to_string(),
        image: "".to_string(),
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
