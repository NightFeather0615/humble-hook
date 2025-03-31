use std::collections::HashMap;

use once_cell::sync::Lazy;
use reqwest::blocking::Client;
use scraper::Html;
use anyhow::{Context, Result};
use serde_json::{json, Value};

use crate::env::ENV;


pub(crate) static HUMBLE_BASE_URL: &'static str = "https://www.humblebundle.com";
pub(crate) static DISCORD_BASE_URL: &'static str = "https://discord.com/api";
pub(crate) static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| reqwest::blocking::Client::new());


pub(crate) fn fetch_webpage(url: &str) -> Result<Html> {
  println!("[ INFO / fetch_webpage ] Fetching `{}`...", url);

  let res = HTTP_CLIENT.get(url).send()?.text()?;

  Ok(Html::parse_document(&res))
}

pub(crate) fn fetch_record(
  channel_id: &str,
  message_id: &str
) -> Result<HashMap<String, i64>> {
  let token = ENV.bot_token.as_str();

  println!("[ INFO / fetch_record ] Fetching record `{channel_id}` -> `{message_id}`...");

  let res = HTTP_CLIENT
    .get(format!("{DISCORD_BASE_URL}/channels/{channel_id}/messages/{message_id}"))
    .header("Authorization", format!("Bot {token}"))
    .send()?;

  let content = serde_json::from_str::<Value>(&res.text()?)?.try_get_string("content")?;

  println!("[ INFO / fetch_record ] Parsing record `{channel_id}` -> `{message_id}`...");

  let records = content
    .replace("```", "")
    .trim()
    .split_terminator("\n")
    .into_iter()
    .map(
      |s| s.split_once(",")
    )
    .collect::<Option<Vec<(&str, &str)>>>()
    .context("Failed split record data")?
    .into_iter()
    .map(
      |p| {
        Ok(
          (
            p.0.to_string(),
            p.1.parse::<i64>()?
          )
        )
      }
    )
    .collect::<Result<Vec<(String, i64)>>>()?;

  Ok(HashMap::from_iter(records))
}

pub(crate) fn update_record(
  new_record: HashMap<String, i64>,
  channel_id: &str,
  message_id: &str
) -> Result<()> {
  let token = ENV.bot_token.as_str();

  println!("[ INFO / update_record ] Encoding record `{channel_id}` -> `{message_id}`...");

  let mut msg = new_record
    .into_iter()
    .map(
      |p| {
        format!(
          "{},{}",
          p.0,
          p.1
        )
      }
    )
    .collect::<Vec<String>>()
    .join("\n");

  msg.insert_str(0, "```");
  msg.push_str("```");

  let data = json!(
    {
      "content": msg
    }
  );

  println!("[ INFO / update_record ] Patching record `{channel_id}` -> `{message_id}`...");

  HTTP_CLIENT
    .patch(
      format!(
        "{DISCORD_BASE_URL}/channels/{channel_id}/messages/{message_id}"
      )
    )
    .header("Authorization", format!("Bot {token}"))
    .header("Content-Type", "application/json")
    .body(data.to_string())
    .send()?;

  Ok(())
}

pub(crate) trait TryGet {
  fn try_get_str(self: &Self, key: &str) -> Result<&str>;
  fn try_get_string(self: &Self, key: &str) -> Result<String>;
  fn try_get_f64(self: &Self, key: &str) -> Result<f64>;
  fn try_get_array(self: &Self, key: &str) -> Result<&Vec<Value>>;
  fn try_as_array(self: &Self) -> Result<&Vec<Value>>;
}

impl TryGet for Value {
  fn try_get_str(self: &Self, key: &str) -> Result<&str> {
    self[key].as_str().context(format!("Missing field `{key}`"))
  }

  fn try_get_string(self: &Self, key: &str) -> Result<String> {
    Ok(self[key].as_str().context(
      format!("Missing field `{key}`"))?.to_string()
    )
  }

  fn try_get_f64(self: &Self, key: &str) -> Result<f64> {
    self[key].as_f64().context(format!("Missing field `{key}`"))
  }

  fn try_get_array(self: &Self, key: &str) -> Result<&Vec<Value>> {
    self[key].as_array().context(format!("Missing field `{key}`"))
  }

  fn try_as_array(self: &Self) -> Result<&Vec<Value>> {
    self.as_array().context(format!("Failed cast as array"))
  }
}
