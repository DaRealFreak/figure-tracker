use std::error::Error;

use kuchiki::traits::TendrilSink;
use kuchiki::NodeRef;

use crate::database::items::Item;
use crate::modules::myfigurecollection::MyFigureCollection;
use crate::modules::InfoModule;

/// small private struct for the not exposed functionality of the InfoModule implementation
struct Info {}

/// the private part of the InfoModule implementation
impl Info {
    /// retrieve the title of the figure from the HTML document of the detail page
    fn get_figure_title_from_doc(doc: &NodeRef) -> Result<String, String> {
        if let Ok(first_element) = doc.select_first("h1 > span.h1-headline span[itemprop='name']") {
            Ok(first_element
                .attributes
                .borrow()
                .get("title")
                .unwrap()
                .to_string())
        } else {
            Err("couldn't find the title".to_string())
        }
    }

    /// retrieve the scale of the figure from the HTML document of the detail page
    fn get_figure_scale_from_doc(doc: &NodeRef) -> Result<String, String> {
        if let Ok(element) = doc.select_first("div.split-right.righter a.item-scale") {
            Ok(element.text_contents())
        } else {
            Err("couldn't find the scale attribute".to_string())
        }
    }

    /// retrieve a generic attribute of the figure in the original language
    /// from the HTML document of the detail page
    fn get_figure_attribute_from_doc_jp(doc: &NodeRef, attr: &str) -> Result<String, String> {
        Info::get_figure_attribute_from_doc(doc, attr, true)
    }

    /// retrieve a generic attribute of the figure in the translated into the english language
    /// from the HTML document of the detail page
    fn get_figure_attribute_from_doc_en(doc: &NodeRef, attr: &str) -> Result<String, String> {
        Info::get_figure_attribute_from_doc(doc, attr, false)
    }

    /// retrieve a generic attribute of the figure from the HTML document of the detail page
    fn get_figure_attribute_from_doc(
        doc: &NodeRef,
        attr: &str,
        use_original: bool,
    ) -> Result<String, String> {
        for element in doc
            .select("div.split-right.righter div.form-field")
            .unwrap()
        {
            let element_node = element.as_node();
            let label = element_node
                .select_first("div.form-label")
                .unwrap()
                .text_contents()
                .to_lowercase();
            if label == attr {
                let value = element_node.select_first("a[href] > span");
                if value.is_err() {
                    return Ok("".to_string());
                }

                if use_original {
                    return Ok(value
                        .unwrap()
                        .attributes
                        .borrow()
                        .get("switch")
                        .unwrap()
                        .to_string());
                }

                return Ok(value.unwrap().text_contents());
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

        let doc = kuchiki::parse_html().one(res.text()?.as_str());

        let mut terms_en: Vec<String> = vec![];
        let mut terms_jp: Vec<String> = vec![];

        for key in vec!["character", "company", "companies"].iter() {
            if let Ok(attr) = Info::get_figure_attribute_from_doc_en(&doc, *key) {
                terms_en.push(attr);
            }

            if let Ok(attr) = Info::get_figure_attribute_from_doc_jp(&doc, *key) {
                terms_jp.push(attr);
            }
        }

        if let Ok(scale) = Info::get_figure_scale_from_doc(&doc) {
            terms_en.push(scale.clone());
            terms_jp.push(scale);
        }

        if let Ok(description) = Info::get_figure_title_from_doc(&doc) {
            item.description = description;
        }

        item.image = format!(
            "https://static.myfigurecollection.net/pics/figure/large/{}.jpg",
            self.get_figure_id(item)?
        );
        item.term_en = terms_en.join(" ");
        item.term_jp = terms_jp.join(" ");

        Ok(())
    }
}
