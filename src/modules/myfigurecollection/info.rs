use std::error::Error;

use scraper::{Html, Selector};

use crate::database::items::Item;
use crate::modules::myfigurecollection::MyFigureCollection;
use crate::modules::InfoModule;

/// small private struct for the not exposed functionality of the InfoModule implementation
struct Info<'a> {
    pub(crate) inner: &'a MyFigureCollection,
}

/// the private part of the InfoModule implementation
impl Info<'_> {
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

    /// retrieve a generic attribute of the figure in the original language
    /// from the HTML document of the detail page
    fn get_figure_attribute_from_doc_jp(document: &Html, attr: &str) -> Result<String, String> {
        Info::get_figure_attribute_from_doc(document, attr, true)
    }

    /// retrieve a generic attribute of the figure in the translated into the english language
    /// from the HTML document of the detail page
    fn get_figure_attribute_from_doc_en(document: &Html, attr: &str) -> Result<String, String> {
        Info::get_figure_attribute_from_doc(document, attr, false)
    }

    /// retrieve a generic attribute of the figure from the HTML document of the detail page
    fn get_figure_attribute_from_doc(
        document: &Html,
        attr: &str,
        use_original: bool,
    ) -> Result<String, String> {
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

                if use_original {
                    return Ok(value.unwrap().value().attr("switch").unwrap().to_string());
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
        let res = self
            .client
            .get(self.get_figure_url(&item)?.as_str())
            .send()?;

        let document = Html::parse_document(&res.text()?);

        let mut terms_en: Vec<String> = vec![];
        let mut terms_jp: Vec<String> = vec![];

        for key in vec!["character", "company"].iter() {
            if let Ok(attr) = Info::get_figure_attribute_from_doc_en(&document, *key) {
                terms_en.push(attr);
            }

            if let Ok(attr) = Info::get_figure_attribute_from_doc_jp(&document, *key) {
                terms_jp.push(attr);
            }
        }

        if let Ok(scale) = Info::get_figure_scale_from_doc(&document) {
            terms_en.push(scale.clone());
            terms_jp.push(scale);
        }

        item.description = Info::get_figure_title_from_doc(&document);
        item.term_en = terms_en.join(" ");
        item.term_jp = terms_jp.join(" ");

        Ok(())
    }
}
