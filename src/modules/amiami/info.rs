use std::convert::TryFrom;
use std::error::Error;
use std::result::Result::Err;

use regex::Regex;
use serde::Deserialize;

use crate::database::items::Item;
use crate::modules::amiami::AmiAmi;
use crate::modules::InfoModule;

/// the search response returns from the API of AmiAmi
#[derive(Deserialize)]
pub(crate) struct ApiSearchResponse {
    search_result: ApiSearchResult,
    items: Vec<ApiItem>,
    #[serde(rename = "_embedded")]
    embedded: ApiEmbedded,
}

/// the results contain only the number of the search results
#[derive(Deserialize)]
pub(crate) struct ApiSearchResult {
    total_results: u64,
}

/// all relevant information regarding the listed items from the API search response
#[derive(Deserialize, Clone)]
pub(crate) struct ApiItem {
    gcode: String,
    gname: String,
    min_price: u64,
    max_price: u64,
    maker_name: String,
    c_price_taxed: u64,
}

/// the _embedded part of the ApiSearchResponse, contains mostly metadata
#[derive(Deserialize)]
pub(crate) struct ApiEmbedded {
    character_names: Vec<ApiCharacterName>,
}

/// there character names contain an ID and a name
#[derive(Deserialize, Clone)]
pub(crate) struct ApiCharacterName {
    id: u64,
    name: String,
}

/// helper implementation for the ApiSearchResponse to process the response even further
impl ApiSearchResponse {
    /// try to find a scale in the item description and return as option if found
    fn get_scale(&self) -> Option<String> {
        let re = Regex::new(r"(1/\d{1,3})").unwrap();
        if re.is_match(self.items[0].gname.as_str()) {
            let test = re.find(self.items[0].gname.as_str()).unwrap();
            return Some(test.as_str().to_string());
        }
        None
    }

    /// retrieve the URL for the item based on the JAN/EAN number
    pub fn get_figure_url(&self) -> String {
        format!(
            "https://www.amiami.com/eng/detail/?gcode={}",
            self.items[0].gcode
        )
    }
}

pub(crate) struct Info<'a> {
    pub(crate) inner: &'a AmiAmi,
}

impl Info<'_> {
    /// retrieve new instance of the Info implementation
    fn new(inner: &AmiAmi) -> Info<'_> {
        Info { inner }
    }

    pub fn search(&self, keyword: String) -> Result<ApiSearchResponse, Box<dyn Error>> {
        let api_url = format!(
            "https://api.amiami.com/api/v1.0/items?pagemax=20\
             &lang=eng&mcode=7000958879&ransu=APEZOBusRNg5WxhFzJqxzTxC9esUCH48\
             &s_keywords={}",
            keyword
        );

        let res = self
            .inner
            .client
            .get(&api_url)
            .header("X-User-Key", "amiami_dev")
            .send()?;

        let deserialized_data: ApiSearchResponse = serde_json::from_str(&res.text()?).unwrap();
        Ok(deserialized_data)
    }
}

/// the InfoModule trait implementation for AmiAmi
impl InfoModule for AmiAmi {
    /// return the module key for logging purposes
    fn get_module_key(&self) -> String {
        AmiAmi::get_module_key()
    }

    /// update the figure details with the extracted information from the search
    fn update_figure_details(&self, mut item: &mut Item) -> Result<(), Box<dyn Error>> {
        let api_response = Info::new(self).search(item.jan.to_string())?;

        match api_response.search_result.total_results {
            0 => return Err(Box::try_from("no search results found").unwrap()),
            1 => (),
            _ => warn!("more than 1 result found for item, extracted information could be wrong"),
        }

        item.description = (&api_response.items[0].gname).to_string();

        let mut terms: Vec<String> = vec![];
        for character in api_response.embedded.character_names.iter() {
            terms.push(character.clone().name);
        }

        terms.push((&api_response.items[0].maker_name).to_string());

        if let Some(scale) = api_response.get_scale() {
            terms.push(scale)
        }

        item.term_en = terms.join(" ");

        Ok(())
    }
}

#[test]
fn test_figure_url() {
    use crate::modules::InfoModule;

    let item = &mut Item {
        id: 0,
        jan: 6_971_995_420_057,
        description: "".to_string(),
        term_en: "".to_string(),
        term_jp: "".to_string(),
        disabled: false,
    };

    assert!(AmiAmi::new().unwrap().update_figure_details(item).is_ok());

    println!("{:?}", item.jan);
    println!("{:?}", item.description);
    println!("{:?}", item.term_en);
}
