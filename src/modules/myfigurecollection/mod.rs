use crate::modules::BaseModule;

pub(crate) struct MyFigureCollection {}

impl BaseModule for MyFigureCollection {
    fn get_module_key(&self) -> &str {
        "myfigurecollection.net"
    }

    fn matches_url(&self, _url: &str) -> bool {
        unimplemented!("not implemented yet")
    }
}