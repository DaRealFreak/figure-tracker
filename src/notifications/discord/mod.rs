use std::error::Error;
use std::fs::File;

use serenity::client::EventHandler;
use serenity::framework::StandardFramework;
use serenity::model::channel::PrivateChannel;
use serenity::model::user::User;
use serenity::Client;

use crate::database::conditions::Condition;
use crate::database::items::Item;
use crate::database::prices::Price;
use crate::http::get_client;
use crate::notifications::Notification;

#[derive(Clone)]
pub(crate) struct DiscordBotData {
    client_token: String,
    user_id: i64,
}

impl DiscordBotData {
    pub fn new(client_token: String, user_id: i64) -> Self {
        DiscordBotData {
            client_token,
            user_id,
        }
    }
}

struct Handler;

impl EventHandler for Handler {}

pub(crate) struct Discord {
    client: Client,
    user: User,
}

impl Discord {
    /// return instance of the Discord notifications which acts as a wrapper for the serenity library
    pub fn new(data: DiscordBotData) -> Self {
        let mut client =
            Client::new(data.client_token, Handler).expect("error creating discord client");
        client.with_framework(StandardFramework::new().configure(|c| c.prefix("!")));

        let user_json = format!(
            "{{
              \"id\": \"{}\",
              \"username\": \"\",
              \"discriminator\": \"1337\",
              \"avatar\": \"\",
              \"verified\": true,
              \"email\": \"\"
            }}",
            data.user_id
        );

        let user = serde_json::from_str(user_json.as_str())
            .expect("couldn't deserialize the discord user");

        Discord { client, user }
    }

    /// send the passed message to the configured user ID
    fn send_message(&self, message: String) -> Result<(), Box<dyn Error>> {
        let private_channel: PrivateChannel = self
            .user
            .create_dm_channel(&self.client.cache_and_http.http)?;
        private_channel.say(&self.client.cache_and_http.http, message)?;

        Ok(())
    }

    /// send the passed image and the passed message to the configured user ID
    fn send_image_message(&self, image_url: String, message: String) -> Result<(), Box<dyn Error>> {
        let private_channel: PrivateChannel = self
            .user
            .create_dm_channel(&self.client.cache_and_http.http)?;

        // download the image to a local temporary file
        let mut tmp = tempfile::NamedTempFile::new()?;
        get_client()?
            .get(image_url.as_str())
            .send()?
            .copy_to(&mut tmp)?;

        // create additional handle for the created temporary file
        // since the tempfile handle won't let us use it again, not too sure why
        let file_handle = File::open(tmp.path())?;

        private_channel.send_files(
            &self.client.cache_and_http.http,
            vec![(&file_handle, "image.png")],
            |m| m.content(message),
        )?;

        // drop tempfile, effectively removing it again
        drop(tmp);

        Ok(())
    }
}

impl Notification for Discord {
    fn notify(&self, item: Item, price: Price, cond: Condition) -> Result<(), Box<dyn Error>> {
        let msg = format!(
            "met search notification on item:\n\
             **{}**\n\
             **{}**\n\
             \n\
             price: **{:.2} {}**\n\
             price with taxes: **{:.2} {}**\n\
             price with shipping and taxes: **{:.2} {}**\n\
             raw price: **{:.2} {}**\n\
             \n\
             item condition: **{:?}**\n\
             \n\
             notification type: **{:?}**\n\
             requested item condition: **{:?}**\n\
             value: **{:.2}**\n\
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
            self.send_image_message(item.image, msg)?;
        } else {
            self.send_message(msg)?;
        }

        Ok(())
    }
}
