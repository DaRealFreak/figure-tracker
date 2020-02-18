use std::error::Error;

use crate::database::items::Item;
use crate::modules::amiami::AmiAmi;
use crate::modules::InfoModule;

impl InfoModule for AmiAmi {
    fn get_module_key(&self) -> String {
        AmiAmi::get_module_key()
    }

    fn update_figure_details(&self, mut item: &mut Item) -> Result<(), Box<dyn Error>> {
        item.description =
            "[Bonus] Houkai 3rd Sakura Yae Chinese Dress Ver. 1/8 Complete Figure(Released)"
                .to_string();
        item.term = "Sakura Yae APEX 1/8".to_string();

        Ok(())
    }
}
