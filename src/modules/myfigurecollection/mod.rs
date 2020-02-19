use std::error::Error;

use crate::database::items::Item;

mod base;
mod info;

pub(crate) struct MyFigureCollection {}

impl MyFigureCollection {
    /// create new instance of MFC
    pub fn new() -> Self {
        MyFigureCollection {}
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
        term: "".to_string(),
        disabled: false,
    };

    assert!(MyFigureCollection::new()
        .update_figure_details(item)
        .is_ok());

    println!("{:?}", item.jan);
    println!("{:?}", item.description);
    println!("{:?}", item.term);
}
