use std::convert::TryFrom;
use std::error::Error;
use std::str::FromStr;

use regex::Regex;

use crate::database::items::Item;
use crate::http::get_client;

mod base;
mod info;

pub(crate) struct MyFigureCollection {
    client: reqwest::blocking::Client,
}

impl MyFigureCollection {
    /// create new instance of MFC
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(MyFigureCollection {
            client: get_client()?,
        })
    }

    pub fn get_module_key() -> String {
        "myfigurecollection.net".to_string()
    }

    /// retrieve the URL for the item based on the JAN/EAN number
    fn get_figure_url(item: &Item) -> String {
        format!(
            "https://myfigurecollection.net/browse.v4.php?barcode={:?}",
            item.jan
        )
    }

    /// retrieve the MFC item ID
    fn get_figure_id(&self, item: &Item) -> Result<u32, Box<dyn Error>> {
        let figure_id_regex =
            Regex::new(r"^https://myfigurecollection.net/item/(?P<item_id>\d+).*$")?;

        let res = self
            .client
            .get(MyFigureCollection::get_figure_url(&item).as_str())
            .send()?;

        // FixMe: this will currently fail on figures with re-releases (f.e. 4934054783441)
        // we should parse the result amount too if the URL is not a match directly
        if !figure_id_regex.is_match(res.url().as_str()) {
            return Err(Box::try_from("no item found by passed JAN").unwrap());
        }

        Ok(figure_id_regex
            .captures(res.url().as_str())
            .and_then(|cap| {
                cap.name("item_id")
                    .map(|item_id| u32::from_str(item_id.as_str()).unwrap())
            })
            .unwrap())
    }
}

#[test]
pub fn test_get_figure_id() {
    let item = &mut Item {
        id: 0,
        jan: 4_571_245_298_836,
        description: "".to_string(),
        term_en: "".to_string(),
        term_jp: "".to_string(),
        disabled: false,
    };

    let mfc = MyFigureCollection {
        client: reqwest::blocking::Client::builder().build().unwrap(),
    };

    println!("{:?}", mfc.get_figure_id(item));
    assert!(mfc.get_figure_id(item).is_ok());
}

#[test]
pub fn test_get_figure_details() {
    use crate::modules::InfoModule;

    let item = &mut Item {
        id: 0,
        jan: 4_571_245_296_405,
        description: "".to_string(),
        term_en: "".to_string(),
        term_jp: "".to_string(),
        disabled: false,
    };

    let mfc = MyFigureCollection {
        client: reqwest::blocking::Client::builder().build().unwrap(),
    };

    assert!(mfc.update_figure_details(item).is_ok());

    println!("JAN: {:?}", item.jan);
    println!("description: {:?}", item.description);
    println!("english term: {:?}", item.term_en);
    println!("japanese term: {:?}", item.term_jp);
}
