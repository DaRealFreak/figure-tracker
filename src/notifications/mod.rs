use std::error::Error;

use crate::configuration::Configuration;
use crate::database::conditions::Condition;
use crate::database::items::Item;
use crate::database::prices::Price;
use crate::notifications::discord::{Discord, DiscordBotData};
use crate::notifications::telegram::{Telegram, TelegramApiData};

pub(crate) mod discord;
pub(crate) mod telegram;

#[derive(Clone)]
pub(crate) struct NotificationManager {
    telegram_data: Option<TelegramApiData>,
    discord_data: Option<DiscordBotData>,
}

pub(crate) trait Notification {
    fn notify(&self, item: Item, price: Price, cond: Condition) -> Result<(), Box<dyn Error>>;
}

impl NotificationManager {
    pub fn new() -> Self {
        let mut manager = NotificationManager {
            telegram_data: None,
            discord_data: None,
        };

        if let Some(api_key) = Configuration::get_telegram_api_key() {
            if let Some(user_id) = Configuration::get_telegram_user_id() {
                manager.telegram_data = Some(TelegramApiData::new(api_key, user_id));
            }
        }

        if let Some(client_token) = Configuration::get_discord_client_token() {
            if let Some(user_id) = Configuration::get_discord_user_id() {
                manager.discord_data = Some(DiscordBotData::new(client_token, user_id));
            }
        }

        manager
    }

    pub fn notify(&self, item: Item, price: Price, cond: Condition) -> Result<(), Box<dyn Error>> {
        if let Some(data) = self.telegram_data.as_ref() {
            Telegram::new(data.clone()).notify(item.clone(), price.clone(), cond.clone())?;
        }

        if let Some(data) = self.discord_data.as_ref() {
            Discord::new(data.clone()).notify(item, price, cond)?;
        }

        Ok(())
    }
}
