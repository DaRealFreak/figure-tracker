use std::convert::TryFrom;
use std::error::Error;
use std::str::FromStr;

use regex::Regex;
use scraper::{Html, Selector};

use crate::currency::CurrencyConversion;
use crate::database::items::Item;
use crate::http::get_client;

mod base;
mod info;

#[derive(Clone)]
pub(crate) struct MyFigureCollection {
    client: reqwest::blocking::Client,
    conversion: CurrencyConversion,
}

impl MyFigureCollection {
    /// create new instance of MFC
    pub fn new(conversion: CurrencyConversion) -> Result<Self, Box<dyn Error>> {
        Ok(MyFigureCollection {
            client: get_client()?,
            conversion,
        })
    }

    pub fn get_module_key() -> String {
        "myfigurecollection.net".to_string()
    }

    /// retrieve the URL for the item based on the JAN/EAN number
    fn get_figure_url(&self, item: &Item) -> Result<String, Box<dyn Error>> {
        match self.get_figure_id(item) {
            Ok(figure_id) => Ok(format!("https://myfigurecollection.net/item/{}", figure_id)),
            Err(err) => Err(err),
        }
    }

    /// retrieve the search URL for the item based on the JAN/EAN number
    fn get_figure_search_url(item: &Item) -> String {
        format!(
            "https://myfigurecollection.net/browse.v4.php?barcode={:?}",
            item.jan
        )
    }

    /// handle search response of the barcode search
    /// figures with multiple matches will also warn about search results
    /// figures with multiple releases and no additional results will function normally
    fn get_figure_id_search_response(document: &Html) -> Result<u32, Box<dyn Error>> {
        let item_selector = Selector::parse("li.listing-item span.item-icon a[href]").unwrap();

        match document.select(&item_selector).count() {
            0 => return Err(Box::try_from("searched figure couldn't be found").unwrap()),
            1 => (),
            _ => warn!("more than 1 result found for item, extracted information could be wrong"),
        }

        if let Some(test) = document.select(&item_selector).next() {
            let rel_link = test.value().attr("href").unwrap();

            let rel_figure_regex = Regex::new(r"/item/(?P<item_id>\d+).*$")?;
            if rel_figure_regex.is_match(rel_link) {
                return Ok(rel_figure_regex
                    .captures(rel_link)
                    .and_then(|cap| {
                        cap.name("item_id")
                            .map(|item_id| u32::from_str(item_id.as_str()).unwrap())
                    })
                    .unwrap());
            }
        }

        Err(Box::try_from("searched figure couldn't be found").unwrap())
    }

    /// retrieve the MFC item ID
    fn get_figure_id(&self, item: &Item) -> Result<u32, Box<dyn Error>> {
        let figure_id_regex =
            Regex::new(r"^https://myfigurecollection.net/item/(?P<item_id>\d+).*$")?;

        let res = self
            .client
            .get(MyFigureCollection::get_figure_search_url(&item).as_str())
            .send()?;

        // move response URL from Url -> &str -> String since retrieving the text will move the value
        let res_url = res.url().as_str().to_string();

        if !figure_id_regex.is_match(res_url.as_str()) {
            let document = Html::parse_document(&res.text()?.as_str());
            return MyFigureCollection::get_figure_id_search_response(&document);
        }

        Ok(figure_id_regex
            .captures(res_url.as_str())
            .and_then(|cap| {
                cap.name("item_id")
                    .map(|item_id| u32::from_str(item_id.as_str()).unwrap())
            })
            .unwrap())
    }
}

#[test]
pub fn test_get_figure_id() {
    use std::collections::BTreeMap;

    let item = &mut Item {
        id: 0,
        jan: 4_571_245_298_836,
        description: "".to_string(),
        term_en: "".to_string(),
        term_jp: "".to_string(),
        disabled: false,
    };

    let item_multiple_releases = &mut Item {
        id: 0,
        jan: 4_934_054_783_441,
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

    assert!(mfc.get_figure_id(item).is_ok());
    assert_eq!(mfc.get_figure_id(item).unwrap(), 740_258);

    assert!(mfc.get_figure_id(item_multiple_releases).is_ok());
    assert_eq!(mfc.get_figure_id(item_multiple_releases).unwrap(), 218_050);
}

#[test]
pub fn test_get_figure_details() {
    use crate::modules::InfoModule;
    use std::collections::BTreeMap;

    let item = &mut Item {
        id: 0,
        jan: 4_571_245_296_405,
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

    assert!(mfc.update_figure_details(item).is_ok());

    println!("JAN: {:?}", item.jan);
    println!("description: {:?}", item.description);
    println!("english term: {:?}", item.term_en);
    println!("japanese term: {:?}", item.term_jp);
}
