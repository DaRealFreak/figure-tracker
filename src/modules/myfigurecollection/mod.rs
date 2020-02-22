use std::error::Error;

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

    /// retrieve the MFC item ID
    fn get_figure_id(item: Item) -> Result<u32, Box<dyn Error>> {
        Ok(0)
    }
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

    assert!(MyFigureCollection::new()
        .unwrap()
        .update_figure_details(item)
        .is_ok());

    println!("JAN: {:?}", item.jan);
    println!("description: {:?}", item.description);
    println!("english term: {:?}", item.term_en);
    println!("japanese term: {:?}", item.term_jp);
}
