use std::error::Error;

use crate::database::items::Item;
use crate::modules::BaseModule;

pub(crate) struct MyFigureCollection {}

impl MyFigureCollection {
    pub fn get_figure_details(&self, item: Item) -> Result<(), Box<dyn Error>> {
        let url = format!("https://myfigurecollection.net/browse.v4.php?barcode={:?}", item.jan);
        let resp = reqwest::blocking::get(&url)?;
        println!("{:#?}", resp.text()?);
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