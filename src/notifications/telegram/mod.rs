use std::error::Error;

use telegram_bot::{Api, FileRef, InputFile, ParseMode, SendMessage, SendPhoto, UserId};
use tokio::runtime::Runtime;

use crate::database::conditions::Condition;
use crate::database::items::Item;
use crate::database::prices::Price;
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
            api: Api::new(data.api_key),
            user_id: data.user_id,
        }
    }

    /// send the passed message to the passed user ID
    fn send_message(&self, message: String) -> Result<(), Box<dyn Error>> {
        let mut send_message = SendMessage::new(UserId::from(self.user_id), message);
        send_message.parse_mode(ParseMode::Html);
        Runtime::new()
            .unwrap()
            .block_on(self.api.send(send_message))?;
        Ok(())
    }

    /// send the passed image with the optional passed caption to the passed user ID
    /// the image path must be available online though and can't be an local file
    fn send_image_message(&self, image_url: String, message: String) -> Result<(), Box<dyn Error>> {
        let mut send_image = SendPhoto::new(
            UserId::from(self.user_id),
            InputFile::from(FileRef::from(image_url)),
        );
        send_image.caption(message);
        send_image.parse_mode(ParseMode::Html);

        Runtime::new()
            .unwrap()
            .block_on(self.api.send(send_image))?;
        Ok(())
    }
}

impl Notification for Telegram {
    fn notify(&self, item: Item, price: Price, cond: Condition) -> Result<(), Box<dyn Error>> {
        let msg = format!(
            "met search notification on item:\n\
             <b>{:?}</b>\n\
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
            self.send_image_message(item.image, msg)?;
        } else {
            self.send_message(msg)?;
        }

        Ok(())
    }
}
