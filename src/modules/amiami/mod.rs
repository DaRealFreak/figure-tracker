mod base;
mod info;

pub(crate) struct AmiAmi {}

impl AmiAmi {
    pub fn new() -> Self {
        AmiAmi {}
    }

    pub fn get_module_key() -> String {
        "amiami.com".to_string()
    }
}
