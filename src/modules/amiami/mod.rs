pub(crate) struct AmiAmi {}

mod base;
mod info;

impl AmiAmi {
    pub fn new() -> Self {
        AmiAmi {}
    }

    pub fn get_module_key() -> String {
        "amiami.com".to_string()
    }
}
