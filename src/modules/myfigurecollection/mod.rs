use std::error::Error;

use scraper::{Html, Selector};

use crate::database::items::Item;
use crate::modules::BaseModule;

pub(crate) struct MyFigureCollection {}

impl MyFigureCollection {
    pub fn get_figure_details(&self, mut item: Item) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "https://myfigurecollection.net/browse.v4.php?barcode={:?}",
            item.jan
        );
        let resp = reqwest::blocking::get(&url)?;

        let document = Html::parse_document(&resp.text()?);
        let title_selector = Selector::parse("h1 > span[itemprop='name']").unwrap();

        let description_element_ref = document.select(&title_selector).next().unwrap();
        item.description = description_element_ref
            .value()
            .attr("title")
            .unwrap()
            .parse()
            .unwrap();

        let item_scale_selector = Selector::parse("div.split-right.righter a.item-scale").unwrap();
        let item_scale_element_ref = document.select(&item_scale_selector).next().unwrap();
        let scale = item_scale_element_ref.text().collect::<Vec<_>>().join("");
        println!("scale: {:?}", scale);

        let attributes_selector =
            Selector::parse("div.split-right.righter div.form > div.form-field").unwrap();
        for element in document.select(&attributes_selector) {
            let label_selector = Selector::parse("div.form-label").unwrap();
            let value_selector = Selector::parse("a[href] > span").unwrap();
            let label = element
                .select(&label_selector)
                .next()
                .unwrap()
                .text()
                .next()
                .unwrap();
            if label == "Character" || label == "Company" {
                let value = element.select(&value_selector).next();
                if value != None {
                    println!(
                        "Attribute: {:?}, Value: {:?}",
                        label,
                        value.unwrap().text().next().unwrap()
                    );
                }
            }
        }

        Ok(())
    }
}

impl BaseModule for MyFigureCollection {
    fn get_module_key(&self) -> &str {
        "myfigurecollection.net"
    }

    fn matches_url(&self, _url: &str) -> bool {
        unimplemented!("not implemented yet")
    }
}

#[test]
pub fn test_get_figure_details() {
    let mfc = MyFigureCollection {};
    mfc.get_figure_details(Item {
        id: 0,
        jan: 4571245296405,
        term: "".parse().unwrap(),
        description: "()".parse().unwrap(),
        disabled: false,
    });
}