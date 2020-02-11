use crate::database::items::Item;
use crate::modules::BaseModule;

pub(crate) struct MyFigureCollection {}

impl MyFigureCollection {
    pub fn get_figure_details(&self, item: Item) {
        println!(
            "checking figure details from URL: https://myfigurecollection.net/browse.v4.php?barcode={:?}",
            item.jan
        )
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