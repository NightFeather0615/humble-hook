use anyhow::Context;
use chrono::{DateTime, Utc};
use scraper::Selector;
use serde_json::Value;
use anyhow::Result;

use crate::utils::{
  fetch_webpage,
  ConvertMarkdown,
  TryGet,
  HUMBLE_BASE_URL
};


#[derive(Debug)]
pub(crate) enum MediaType {
  Game,
  EBook,
  Software,
  Unknown
}

impl MediaType {
  fn from_str(value: &str) -> Self {
    match value {
      "game" => Self::Game,
      "ebook" => Self::EBook,
      "software" => Self::Software,
      _ => Self::Unknown
    }
  }
}

#[derive(Debug)]
pub(crate) struct Product {
  pub(crate) author: String,
  pub(crate) name: String,
  pub(crate) machine_name: String,
  pub(crate) _media_type: MediaType,
  pub(crate) start_date: DateTime<Utc>,
  pub(crate) end_date: DateTime<Utc>,
  pub(crate) _description: String,
  pub(crate) detailed_blurb: String,
  pub(crate) _blurb: String,
  pub(crate) _short_blurb: String,
  pub(crate) worth: i32,
  pub(crate) high_price: i32,
  pub(crate) low_price: i32,
  pub(crate) product_url: String,
  pub(crate) _logo_image_url: String,
  pub(crate) thumbnail_image_url: String,
  pub(crate) item_names: Vec<String>,
  pub(crate) charity_names: Vec<String>
}

impl Product {
  pub(crate) fn from_json(json: &Value) -> Result<Self> {
    println!("[INFO] Parsing product JSON...");

    let product_url = format!(
      "{HUMBLE_BASE_URL}{}",
      json.try_get_string("product_url")?
    );
    let document = fetch_webpage(&product_url)?;

    let selector = Selector::parse(
      "script#webpack-bundle-page-data"
    ).expect("Failed init selector `script#webpack-bundle-page-data`");

    let detailed_json = serde_json::from_str::<Value>(
      document
        .select(&selector)
        .next()
        .context("Cannot get page JSON data")?
        .text()
        .next()
        .context("Cannot get JSON text")?
    )?;

    let bundle_data = &detailed_json["bundleData"];

    let tier_order = bundle_data.try_get_array("tier_order")?;
    let highest_tier_key = tier_order
      .first()
      .context("Cannot get first tier")?
      .as_str()
      .context("Cannot get tier key")?;
    let lowest_tier_key = tier_order
      .last()
      .context("Cannot get last tier")?
      .as_str()
      .context("Cannot get tier key")?;

    let tier_pricing_data = &bundle_data["tier_pricing_data"];

    let highest_tier_data = &tier_pricing_data[highest_tier_key]["price|money"].try_get_f64("amount")?;
    let lowest_tier_data = &tier_pricing_data[lowest_tier_key]["price|money"].try_get_f64("amount")?;

    let tier_display_data = &bundle_data["tier_display_data"];

    let item_machine_names = tier_display_data[highest_tier_key].try_get_array("tier_item_machine_names")?;
    let mut item_names = Vec::new();

    let tier_item_data = &bundle_data["tier_item_data"];

    for machine_name in item_machine_names {
      let machine_name = machine_name.as_str().context("Cannot get title name")?;

      item_names.push(
        tier_item_data[machine_name].try_get_str("human_name")?.to_string()
      );
    }

    let charity_data = &bundle_data["charity_data"];

    let charity_machine_names = charity_data.try_get_array("charity_item_machine_names")?;
    let mut charity_names = Vec::new();

    let charity_items = &charity_data["charity_items"];

    for machine_name in charity_machine_names {
      let machine_name = machine_name.as_str().context("Cannot get title name")?;

      charity_names.push(
        charity_items[machine_name].try_get_str("human_name")?.to_string()
      );
    }

    let basic_data = &bundle_data["basic_data"];

    Ok(
      Self {
        author: json.try_get_string("author")?,
        name: json.try_get_string("tile_name")?,
        machine_name: json.try_get_string("machine_name")?,
        _media_type: MediaType::from_str(basic_data.try_get_str("media_type")?),
        start_date: (json.try_get_string("start_date|datetime")? + "Z").parse()?,
        end_date: (json.try_get_string("end_date|datetime")? + "Z").parse()?,
        _description: basic_data.try_get_string("description")?,
        detailed_blurb: json.try_get_string("detailed_marketing_blurb")?.to_md(),
        _blurb: json.try_get_string("marketing_blurb")?.to_md(),
        _short_blurb: json.try_get_string("short_marketing_blurb")?.to_md(),
        worth: basic_data["msrp|money"].try_get_f64("amount")?.round() as i32,
        high_price: (*highest_tier_data).round() as i32,
        low_price: (*lowest_tier_data).round() as i32,
        product_url: product_url,
        _logo_image_url: json.try_get_string("tile_logo")?,
        thumbnail_image_url: json.try_get_string("high_res_tile_image")?,
        item_names: item_names,
        charity_names: charity_names
      }
    )
  }
}
