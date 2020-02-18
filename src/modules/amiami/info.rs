use std::convert::TryFrom;
use std::error::Error;

use reqwest::blocking::Client;
use serde_json::Value;

use crate::database::items::Item;
use crate::modules::amiami::AmiAmi;
use crate::modules::InfoModule;

struct Info {}

impl Info {
    /// retrieve the URL for the item based on the JAN/EAN number
    fn get_figure_url(item: &Item) -> Result<String, Box<dyn Error>> {
        let api_url = format!(
            "https://api.amiami.com/api/v1.0/items?pagemax=20\
            &lang=eng&mcode=7000958879&ransu=APEZOBusRNg5WxhFzJqxzTxC9esUCH48\
            &s_keywords={:?}",
            item.jan
        );

        let client = Client::new();
        let res = client
            .get(&api_url)
            .header("X-User-Key", "amiami_dev")
            .send()?;

        let v: Value = serde_json::from_str(&res.text()?)?;
        let total_results = &v["search_result"]["total_results"];

        match total_results {
            Value::Number(total_results) => match total_results.as_u64().unwrap() {
                0 => return Err(Box::try_from("no search results found").unwrap()),
                1 => (),
                _ => warn!("more than 1 result found for item, info could be wrong"),
            },
            _ => error!("unexpected API response"),
        }

        Ok(format!(
            "https://www.amiami.com/eng/detail/?gcode={}",
            v["items"][0]["gcode"].as_str().unwrap()
        ))
    }
}

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

#[test]
fn test_figure_url() {
    use crate::modules::InfoModule;

    let item = &mut Item {
        id: 0,
        jan: 6971995420057,
        description: "".to_string(),
        term: "".to_string(),
        disabled: false,
    };

    match Info::get_figure_url(item) {
        Ok(figure_url) => println!("{}", figure_url),
        Err(err) => println!("{}", err.description()),
    }
    assert!(Info::get_figure_url(item).is_ok());

    assert!(AmiAmi::new().update_figure_details(item).is_ok());

    println!("{:?}", item.jan);
    println!("{:?}", item.description);
    println!("{:?}", item.term);
}
