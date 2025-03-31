use anyhow::Result;
use once_cell::sync::Lazy;


pub(crate) struct EnvConfig {
  pub(crate) bot_token: String,
  pub(crate) game_channel_id: String,
  pub(crate) game_message_id: String,
  pub(crate) ebook_channel_id: String,
  pub(crate) ebook_message_id: String,
  pub(crate) software_channel_id: String,
  pub(crate) software_message_id: String
}

pub(crate) static ENV: Lazy<EnvConfig> = Lazy::new(
  || EnvConfig::init().expect("Failed init `EnvConfig`")
);

impl EnvConfig {
  fn init() -> Result<Self> {
    println!("[ INFO / EnvConfig::init ] Loading environment variables...");

    dotenvy::from_filename_override(".env")?;
    
    Ok(
      Self {
        bot_token: std::env::var("BOT_TOKEN")?,
        game_channel_id: std::env::var("GAME_CHANNEL_ID")?,
        game_message_id: std::env::var("GAME_MESSAGE_ID")?,
        ebook_channel_id: std::env::var("EBOOK_CHANNEL_ID")?,
        ebook_message_id: std::env::var("EBOOK_MESSAGE_ID")?,
        software_channel_id: std::env::var("SOFTWARE_CHANNEL_ID")?,
        software_message_id: std::env::var("SOFTWARE_MESSAGE_ID")?
      }
    )
  }
}
