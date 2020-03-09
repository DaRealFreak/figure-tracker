use std::error::Error;

use telegram_bot::{Api, FileRef, InputFile, SendMessage, SendPhoto, UserId};
use tokio::runtime::Runtime;

struct Telegram {
    core: Runtime,
    api: Api,
}

impl Telegram {
    /// return instance of the Telegram Notifications which acts as a wrapper for the telegram-bot library
    /// it also handles the future promises and you can use it in a blocking context
    pub fn new(api_key: &str) -> Telegram {
        Telegram {
            core: Runtime::new().unwrap(),
            api: Api::new(api_key.to_string()),
        }
    }

    /// send the passed message to the passed user ID
    pub fn send_message(&mut self, user_id: i64, message: &str) -> Result<(), Box<dyn Error>> {
        let send_message = SendMessage::new(UserId::from(user_id), message);
        self.core.block_on(self.api.send(send_message))?;
        Ok(())
    }

    /// send the passed image with the optional passed caption to the passed user ID
    /// the image path must be available online though and can't be an local file
    pub fn send_image_message(
        &mut self,
        user_id: i64,
        image_url: &str,
        message: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let mut send_image = SendPhoto::new(
            UserId::from(user_id),
            InputFile::from(FileRef::from(image_url)),
        );
        if let Some(message) = message {
            send_image.caption(message);
        }

        self.core.block_on(self.api.send(send_image))?;
        Ok(())
    }
}
