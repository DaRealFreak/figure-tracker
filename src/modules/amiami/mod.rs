use crate::database::items::Item;

pub(crate) struct AmiAmi {}

mod info;

impl AmiAmi {
    pub fn new() -> Self {
        AmiAmi {}
    }

    pub fn get_module_key() -> String {
        "amiami.com".to_string()
    }

    /// retrieve the URL for the item based on the JAN/EAN number
    fn get_figure_url(item: &Item) -> String {
        println!("{:?}", item.jan);
        "ToDo".to_string()
    }
}
