use std::error::Error;

use serde::Deserialize;

use crate::currency::guesser::CurrencyGuesser;
use crate::currency::SupportedCurrency;
use crate::database::items::{Item, ItemConditions};
use crate::database::prices::Price;
use crate::modules::solarisjapan::SolarisJapan;
use crate::modules::{BaseModule, Prices};

/// the search response returns from the API of SolarisJapan
#[derive(Deserialize)]
struct ApiSearchResponse {
    results: Vec<ApiSearchResults>,
}

#[derive(Deserialize)]
struct ApiSearchResults {
    #[serde(rename = "hits")]
    items: Vec<ApiSearchItem>,
}

#[derive(Deserialize, Clone)]
struct ApiSearchItem {
    handle: String,
    #[serde(rename = "_highlightResult")]
    matches: HighlightResult,
}

#[derive(Deserialize, Clone)]
struct HighlightResult {
    ean: SearchMatch,
}

#[derive(Deserialize, Clone)]
struct SearchMatch {
    #[serde(rename = "matchLevel")]
    match_level: String,
}

impl ApiSearchItem {
    pub fn get_url(&self) -> String {
        format!("https://solarisjapan.com/products/{}", self.handle)
    }

    pub fn get_info(&self, client: reqwest::blocking::Client) -> Result<ApiInfo, Box<dyn Error>> {
        let info_url = format!("{}.json", self.get_url());

        let res = client.get(info_url.as_str()).send()?;
        let info: ApiInfo = serde_json::from_str(&res.text()?).unwrap();

        Ok(info)
    }
}

#[derive(Deserialize)]
struct ApiInfo {
    product: Product,
}

#[derive(Deserialize)]
struct Product {
    variants: Vec<ProductVariant>,
}

#[derive(Deserialize)]
struct ProductVariant {
    title: String,
    price: String,
}

struct Base {}

impl Base {
    fn get_closest_search_result(search_response: ApiSearchResponse) -> Option<ApiSearchItem> {
        for search_result in search_response.results[0].items.iter() {
            if search_result.matches.ean.match_level == "full" {
                return Some(search_result.clone());
            }
        }

        if !search_response.results.is_empty() {
            warn!(
                "[{}] - no exact match could be found, using closest search result",
                SolarisJapan::get_module_key()
            );
            return Some(search_response.results[0].items[0].clone());
        }

        None
    }
}

impl BaseModule for SolarisJapan {
    /// retrieve the module key
    fn get_module_key(&self) -> String {
        SolarisJapan::get_module_key()
    }

    /// retrieve the lowest price for new and used condition
    fn get_lowest_prices(&self, item: &Item) -> Result<Prices, Box<dyn Error>> {
        let api_url = "https://zzb7273v5r-dsn.algolia.net/1/indexes/*/queries\
                       ?x-algolia-api-key=159b58d793c7a4ebd5928c6b8c100941\
                       &x-algolia-application-id=ZZB7273V5R\
                       &x-algolia-agent=Algolia%20for%20vanilla%20JavaScript%203.10.2";

        let data = format!("{{
            \"requests\": [
                {{
                    \"indexName\": \"ShopifyProduct\",
                    \"params\": \"query={}&query={}&page=0&facets=%5B%22type%22%2C%22Figure%22%2C%22Linen%22%2C%22Game%22%2C%22Book%22%2C%22stock%22%2C%22Video%22%2C%22Music%22%2C%22Merchandise%22%5D&tagFilters=\"
                }}
            ]
        }}", item.jan, item.jan);

        let mut prices = Prices {
            used: None,
            new: None,
        };

        let res = self.client.post(api_url).body(data).send()?;
        let deserialized_data: ApiSearchResponse = serde_json::from_str(&res.text()?).unwrap();

        if let Some(search_result) = Base::get_closest_search_result(deserialized_data) {
            if let Ok(info) = search_result.get_info(self.clone().client) {
                for variant in info.product.variants {
                    // not available, skip here
                    if variant.price.as_str() == "0" {
                        continue;
                    }

                    let mut price = Price::new(
                        item.clone(),
                        CurrencyGuesser::get_currency_value(variant.price)?,
                        SupportedCurrency::JPY,
                        search_result.get_url(),
                        SolarisJapan::get_module_key(),
                        ItemConditions::New,
                    );

                    match variant.title.as_str() {
                        "Brand New" => {
                            prices.new = Some(price);
                        }
                        "Pre Owned" => {
                            price.condition = ItemConditions::Used;
                            prices.used = Some(price);
                        }
                        _ => warn!("unknown condition: {}", variant.title),
                    }
                }
            }
        }

        Ok(prices)
    }
}

#[test]
pub fn test_get_lowest_prices() {
    let item = &mut Item {
        id: 0,
        jan: 4_934_054_783_441,
        description: "".to_string(),
        term_en: "".to_string(),
        term_jp: "".to_string(),
        disabled: false,
    };

    let solaris = SolarisJapan {
        client: reqwest::blocking::Client::builder().build().unwrap(),
    };

    if let Err(err) = solaris.get_lowest_prices(item) {
        println!("{}", err)
    }
}
