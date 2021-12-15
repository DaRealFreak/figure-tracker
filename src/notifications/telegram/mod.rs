use std::error::Error;
use std::path::Path;

use frankenstein::api_params::{SendMessageParamsBuilder, SendPhotoParamsBuilder};
use frankenstein::TelegramApi;
use frankenstein::{Api, InputFile};
use tempfile::tempdir;

use crate::database::conditions::Condition;
use crate::database::items::Item;
use crate::database::prices::Price;
use crate::http::get_client;
use crate::notifications::Notification;

#[derive(Clone)]
pub(crate) struct TelegramApiData {
    api_key: String,
    user_id: i64,
}

impl TelegramApiData {
    pub fn new(api_key: String, user_id: i64) -> Self {
        TelegramApiData { api_key, user_id }
    }
}

pub(crate) struct Telegram {
    api: Api,
    user_id: i64,
}

impl Telegram {
    /// return instance of the Telegram Notifications which acts as a wrapper for the telegram-bot library
    /// it also handles the future promises and you can use it in a blocking context
    pub fn new(data: TelegramApiData) -> Telegram {
        Telegram {
            api: Api::new(&*data.api_key),
            user_id: data.user_id,
        }
    }

    /// send the passed message to the passed user ID
    fn send_message(&self, message: String) -> Result<(), frankenstein::Error> {
        let send_message_params = SendMessageParamsBuilder::default()
            .chat_id(self.user_id)
            .text(&*message)
            .parse_mode("html")
            .build()
            .unwrap();

        self.api.send_message(&send_message_params)?;
        Ok(())
    }

    /// send the passed image with the optional passed caption to the passed user ID
    /// the image path must be available online though and can't be an local file
    fn send_image_message(
        &self,
        image_url: String,
        message: String,
    ) -> Result<(), frankenstein::Error> {
        // get file content from URL
        let resp = get_client()
            .expect("unable to build client")
            .get(&*image_url)
            .send()
            .expect("request failed");
        let body = resp.bytes().expect("body invalid");

        // create tmp dir
        let dir = tempdir().expect("unable to create tmp dir");
        // retrieve file name from URL and create the file path in the tmp dir
        let ancestors = Path::new(&*image_url).file_name().unwrap();
        let file_path = dir.path().join(ancestors.to_str().unwrap());

        // download file
        std::fs::write(file_path.as_path(), &body).expect("unable to download file");

        let file = frankenstein::api_params::File::InputFile(InputFile { path: file_path });
        let params = SendPhotoParamsBuilder::default()
            .chat_id(self.user_id)
            .photo(file)
            .caption(&*message)
            .parse_mode("html")
            .build()
            .unwrap();

        self.api.send_photo(&params)?;

        // drop tmp dir again to delete all files in it
        dir.close().expect("unable to close tmp dir");

        Ok(())
    }
}

impl Notification for Telegram {
    fn notify(&self, item: Item, price: Price, cond: Condition) -> Result<(), Box<dyn Error>> {
        let msg = format!(
            "met search notification on item:\n\
             <b>{}</b>\n\
             <b>{}</b>\n\
             \n\
             price: <b>{:.2} {}</b>\n\
             price with taxes: <b>{:.2} {}</b>\n\
             price with shipping and taxes: <b>{:.2} {}</b>\n\
             raw price: <b>{:.2} {}</b>\n\
             \n\
             item condition: <b>{:?}</b>\n\
             \n\
             notification type: <b>{:?}</b>\n\
             requested item condition: <b>{:?}</b>\n\
             value: <b>{:.2}</b>\n\
             \n\
             link: {}",
            item.description,
            item.jan,
            price.converted_price,
            price.converted_currency,
            price.get_converted_taxed(),
            price.converted_currency,
            price.get_converted_total(),
            price.converted_currency,
            price.price,
            price.currency,
            price.condition,
            cond.condition_type,
            cond.item_condition,
            cond.value,
            price.url,
        );
        if !item.image.is_empty() {
            self.send_image_message(item.image, msg)
                .expect("unable to send image message");
        } else {
            self.send_message(msg).expect("unable to send message");
        }

        Ok(())
    }
}

#[test]
pub fn test_image_message() {
    let api_data = Some(TelegramApiData::new("token".to_string(), 0));

    if let Some(data) = api_data.as_ref() {
        Telegram::new(data.clone())
            .send_image_message(
                "https://static.myfigurecollection.net/pics/figure/large/740258.jpg".to_string(),
                "<b>test</b> image".to_string(),
            )
            .expect("failure sending image message");
    }
}
