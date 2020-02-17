use std::error::Error;

use chrono::Utc;
use scraper::{Html, Selector};

use crate::database::items::{Item, ItemConditions};
use crate::database::prices::Price;
use crate::modules::{BaseModule, Prices};

pub(crate) struct MyFigureCollection {}

impl MyFigureCollection {
    /// retrieve the URL for the item based on the JAN/EAN number
    fn get_figure_url(item: Item) -> String {
        format!(
            "https://myfigurecollection.net/browse.v4.php?barcode={:?}",
            item.jan
        )
    }

    /// retrieve the title of the figure from the HTML document of the detail page
    fn get_figure_title_from_doc(document: Html) -> String {
        let selector = Selector::parse("h1 > span[itemprop='name']").unwrap();

        let description_element_ref = document.select(&selector).next().unwrap();
        description_element_ref
            .value()
            .attr("title")
            .unwrap()
            .parse()
            .unwrap()
    }

    /// retrieve the scale of the figure from the HTML document of the detail page
    fn get_figure_scale_from_doc(document: Html) -> String {
        let item_scale_selector = Selector::parse("div.split-right.righter a.item-scale").unwrap();
        let item_scale_element_ref = document.select(&item_scale_selector).next().unwrap();
        item_scale_element_ref.text().collect::<Vec<_>>().join("")
    }

    /// retrieve a generic attribute of the figure from the HTML document of the detail page
    fn get_figure_attribute_from_doc(document: Html, attr: &str) -> Result<String, String> {
        let selector = Selector::parse("div.split-right.righter div.form-field").unwrap();
        for element in document.select(&selector) {
            let label_selector = Selector::parse("div.form-label").unwrap();
            let value_selector = Selector::parse("a[href] > span").unwrap();
            let label = element
                .select(&label_selector)
                .next()
                .unwrap()
                .text()
                .next()
                .unwrap()
                .to_lowercase();
            if label == attr {
                let value = element.select(&value_selector).next();
                if value == None {
                    return Ok("".to_string());
                }
                return Ok(value.unwrap().text().next().unwrap().to_string());
            }
        }

        Err(format!(
            "attribute {:?} not found in the figure details",
            attr
        ))
    }

    /// update the figure details
    pub fn update_figure_details(mut item: &mut Item) -> Result<(), Box<dyn Error>> {
        let resp = reqwest::blocking::get(&MyFigureCollection::get_figure_url(item.clone()))?;
        let document = Html::parse_document(&resp.text()?);

        let mut terms: Vec<String> = vec![];
        for key in vec!["character", "company"].iter() {
            let attr = MyFigureCollection::get_figure_attribute_from_doc(document.clone(), *key);
            if attr.is_ok() {
                terms.push(attr.unwrap());
            }
        }

        terms.push(MyFigureCollection::get_figure_scale_from_doc(
            document.clone(),
        ));

        item.description = MyFigureCollection::get_figure_title_from_doc(document.clone());
        item.term = terms.join(" ");

        Ok(())
    }

    /// retrieve the MFC item ID
    fn get_figure_id(item: Item) -> Result<u32, Box<dyn Error>> {
        Ok(0)
    }
}

impl BaseModule for MyFigureCollection {
    /// generate instance of Module
    fn new() -> Self {
        MyFigureCollection {}
    }

    fn get_module_key(&self) -> &str {
        "myfigurecollection.net"
    }

    fn get_lowest_prices(&self, item: Item) -> Result<Prices, Box<dyn Error>> {
        let figure_id = MyFigureCollection::get_figure_id(item)?;
        Ok(Prices {
            new: Option::from(Price {
                id: None,
                price: 10.02,
                url: "".to_string(),
                module: self.get_module_key().to_string(),
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
pub fn test_get_figure_details() {
    let item = &mut Item {
        id: 0,
        jan: 4571245296405,
        description: "".to_string(),
        term: "".to_string(),
        disabled: false,
    };

    assert!(MyFigureCollection::update_figure_details(item).is_ok());

    println!("{:?}", item.jan);
    println!("{:?}", item.description);
    println!("{:?}", item.term);
}
