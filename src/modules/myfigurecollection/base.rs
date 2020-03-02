use std::borrow::Borrow;
use std::error::Error;

use scraper::{ElementRef, Html, Selector};

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
    /// retrieve all item listing on the current page
    fn get_sales(doc: &Html) -> Vec<ElementRef> {
        let element_selector = Selector::parse("ul.listing li.listing-item").unwrap();
        doc.select(&element_selector).collect()
    }

    /// extract sales from url and navigate through possibly multiple pages
    fn get_sales_from_url(&self, search_url: String) -> Result<Vec<Price>, Box<dyn Error>> {
        let mut sales = vec![];
        let used_currency = Configuration::get_used_currency();
        let mut res = self.inner.client.get(search_url.as_str()).send()?;

        loop {
            let doc = Html::parse_document(res.text()?.as_str());
            for element in Base::get_sales(&doc) {
                let currency = Base::get_sale_currency(&element);
                let price = Base::get_sale_price(&element);
                let sale_url = Base::get_sale_url(&element);

                if let Some(currency) = CurrencyGuesser::new().guess_currency(currency) {
                    if let Ok(price_value) = CurrencyGuesser::get_currency_value(price.clone()) {
                        let mut price = Price::new(
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

            if let Some(next_page_url) = Base::has_next_page(&doc) {
                res = self.inner.client.get(next_page_url.as_str()).send()?;
            } else {
                break;
            }
        }

        Ok(sales)
    }

    /// retrieve all sales
    fn get_all_sales(&self, figure_id: u32) -> Result<Vec<Price>, Box<dyn Error>> {
        let search_url = format!(
            "https://myfigurecollection.net/classified.php?type=0&itemId={}",
            figure_id
        );

        self.get_sales_from_url(search_url)
    }

    /// retrieve new sales
    fn get_new_sales(&self, figure_id: u32) -> Result<Vec<Price>, Box<dyn Error>> {
        let search_url = format!(
            "https://myfigurecollection.net/classified.php?type=0&itemId={}&isMIB=1",
            figure_id
        );

        self.get_sales_from_url(search_url)
    }

    /// retrieve the next page of the navigation
    /// if no next page exists it'll return None
    fn has_next_page(doc: &Html) -> Option<String> {
        let page_selector = Selector::parse("nav.listing-count-pages > a.nav-next[href]").unwrap();
        if doc.select(&page_selector).count() == 0 {
            return None;
        }

        Some(
            doc.select(&page_selector)
                .next()
                .unwrap()
                .value()
                .attr("href")
                .unwrap()
                .to_string(),
        )
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

    fn get_sale_url(element: &ElementRef<'a>) -> String {
        let sale_selector = Selector::parse("a.tbx-tooltip[href*='classified']").unwrap();
        element
            .select(&sale_selector)
            .next()
            .unwrap()
            .value()
            .attr("href")
            .unwrap()
            .to_string()
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
        let all_sales = Base { inner: self }.get_all_sales(figure_id.clone())?;
        let new_sales = Base { inner: self }.get_new_sales(figure_id)?;

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
