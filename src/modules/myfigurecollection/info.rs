use std::convert::TryFrom;
use std::error::Error;

use regex::Regex;
use reqwest::blocking::get;
use scraper::{Html, Selector};

use crate::database::items::Item;
use crate::modules::myfigurecollection::MyFigureCollection;
use crate::modules::InfoModule;

/// small private struct for the not exposed functionality of the InfoModule implementation
struct Info {}

/// the private part of the InfoModule implementation
impl Info {
    /// retrieve the URL for the item based on the JAN/EAN number
    fn get_figure_url(item: &Item) -> String {
        format!(
            "https://myfigurecollection.net/browse.v4.php?barcode={:?}",
            item.jan
        )
    }

    /// retrieve the title of the figure from the HTML document of the detail page
    fn get_figure_title_from_doc(document: &Html) -> String {
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
    fn get_figure_scale_from_doc(document: &Html) -> Result<String, String> {
        let item_scale_selector = Selector::parse("div.split-right.righter a.item-scale").unwrap();
        let item_scale_element_ref = document.select(&item_scale_selector).next();
        if let Some(element) = item_scale_element_ref {
            Ok(element.text().collect::<Vec<_>>().join(""))
        } else {
            Err("couldn't find the scale attribute".to_string())
        }
    }

    /// retrieve a generic attribute of the figure from the HTML document of the detail page
    fn get_figure_attribute_from_doc(document: &Html, attr: &str) -> Result<String, String> {
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
}

impl InfoModule for MyFigureCollection {
    fn get_module_key(&self) -> String {
        MyFigureCollection::get_module_key()
    }

    /// update the figure details
    fn update_figure_details(&self, mut item: &mut Item) -> Result<(), Box<dyn Error>> {
        let res = get(&Info::get_figure_url(&item))?;

        if !Regex::new(r"^https://myfigurecollection.net/item/")?.is_match(res.url().as_str()) {
            return Err(Box::try_from("no item found by passed JAN").unwrap());
        }

        let document = Html::parse_document(&res.text()?);

        let mut terms: Vec<String> = vec![];
        for key in vec!["character", "company"].iter() {
            let attr = Info::get_figure_attribute_from_doc(&document, *key);
            if let Ok(attr) = attr {
                terms.push(attr);
            }
        }

        terms.push(Info::get_figure_scale_from_doc(&document)?);

        item.description = Info::get_figure_title_from_doc(&document);
        item.term_en = terms.join(" ");

        Ok(())
    }
}
