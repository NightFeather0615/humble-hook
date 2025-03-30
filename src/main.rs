use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Utc;
use scraper::Selector;
use serde_json::Value;

use crate::embed::EmbedMessage;
use crate::product::Product;
use crate::utils::{
  fetch_webpage,
  fetch_record,
  update_record,
  TryGet,
  HUMBLE_BASE_URL
};
use crate::env::ENV;


mod product;
mod embed;
mod utils;
mod env;


fn process_category(
  data: &Value,
  channel_id: &str,
  message_id: &str
) -> Result<()> {
  let products = data
    .try_as_array()?
    .into_iter()
    .map(
      |i| Product::from_json(i)
    )
    .collect::<Result<Vec<Product>>>()?;

  let mut records = fetch_record(
    channel_id,
    message_id
  )?;

  for product in products {
    if records.contains_key(&product.machine_name) {
      continue;
    }

    records.insert(
      product.machine_name.clone(),
      product.end_date.timestamp()
    );

    let embed = EmbedMessage::from_product(product);

    embed.send(channel_id)?;

    sleep(Duration::from_secs(1));
  }

  let now = Utc::now().timestamp();

  let filtered_records = records.clone()
    .into_iter()
    .filter(
      |p| now < p.1
    )
    .collect::<HashMap<String, i64>>();

  update_record(
    filtered_records,
    channel_id,
    message_id
  )?;

  Ok(())
}

fn main() -> Result<()> {
  println!("[INFO] Fetching all bundle info...");
  let document = fetch_webpage(&format!("{HUMBLE_BASE_URL}/bundles"))?;

  let selector = Selector::parse(
    "script#landingPage-json-data"
  ).expect("Failed init selector `script#landingPage-json-data`");

  println!("[INFO] Parsing all bundle info...");

  let bundle_datas: Value = serde_json::from_str(
    document
      .select(&selector)
      .next()
      .context("Cannot get page JSON data")?
      .text()
      .next()
      .context("Cannot get JSON text")?
  )?;

  println!("[INFO] Processing games...");

  process_category(
    &bundle_datas["data"]["games"]["mosaic"][0]["products"],
    &ENV.game_channel_id,
    &ENV.game_message_id
  )?;

  println!("[INFO] Processing books...");

  process_category(
    &bundle_datas["data"]["books"]["mosaic"][0]["products"],
    &ENV.ebook_channel_id,
    &ENV.ebook_message_id
  )?;

  println!("[INFO] Processing softwares...");

  process_category(
    &bundle_datas["data"]["software"]["mosaic"][0]["products"],
    &ENV.software_channel_id,
    &ENV.software_message_id
  )?;

  Ok(())
}
