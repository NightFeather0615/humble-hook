use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde_json::{json, Value};
use anyhow::Result;

use crate::utils::{
  TryGet,
  HTTP_CLIENT,
  DISCORD_BASE_URL
};
use crate::env::ENV;
use crate::product::Product;


pub(crate) struct EmbedField {
  name: &'static str,
  value: String,
  inline: bool
}

impl EmbedField {
  pub(crate) fn new(
    name: &'static str,
    value: String,
    inline: bool
  ) -> Self {
    Self { name, value, inline }
  }
}

pub(crate) struct EmbedMessage {
  title: String,
  description: String,
  url: String,
  timestamp: DateTime<Utc>,
  image_url: String,
  footer: String,
  fields: Vec<EmbedField>
}

impl EmbedMessage {
  pub(crate) fn from_product(product: Product) -> Self {
    println!("[INFO] Encoding product embed...");

    let mut fields = Vec::new();

    if product.low_price == product.high_price {
      fields.push(
        EmbedField::new(
          "Pricing",
          format!(
            "{}$ (MSRP {}$)",
            product.low_price,
            product.worth
          ),
          true
        )
      );
    } else {
      fields.push(
        EmbedField::new(
          "Pricing",
          format!(
            "{}$ ~ {}$ (MSRP {}$)",
            product.low_price,
            product.high_price,
            product.worth
          ),
          true
        )
      );
    }

    fields.push(
      EmbedField::new(
        "Offer ends",
        format!(
          "<t:{}:R>",
          product.end_date.timestamp()
        ),
        true
      )
    );

    if !product.item_names.is_empty() {
      let mut items_md = String::new();

      for item in product.item_names.into_iter() {
        let md = format!("- {item}\n");
        if (items_md.len() + md.len()) >= 1024 {
          items_md.push_str("- ...");
          break;
        }
        items_md.push_str(&md);
      }
  
      fields.push(
        EmbedField::new(
          "Items",
          items_md.trim_end().to_string(),
          false
        )
      );
    }

    if !product.charity_names.is_empty() {
      let mut charities_md = String::new();

      for charity in product.charity_names.into_iter() {
        let md = format!("- {charity}\n");
        if (charities_md.len() + md.len()) >= 1024 {
          charities_md.push_str("- ...");
          break;
        }
        charities_md.push_str(&md);
      }
  
      fields.push(
        EmbedField::new(
          "Charities",
          charities_md.trim_end().to_string(),
          false
        )
      );
    }

    Self {
      title: product.name,
      description: product.detailed_blurb,
      url: product.product_url,
      timestamp: product.start_date,
      image_url: product.thumbnail_image_url,
      footer: product.author,
      fields: fields
    }
  }

  pub(crate) fn send(self: &Self, channel_id: &str) -> Result<()> {
    let json = json!(
      {
        "embeds": [
          {
            "title": self.title,
            "url": self.url,
            "description": self.description,
            "timestamp": self.timestamp.to_rfc3339(),
            "color": 13313833,
            "image": {
              "url": self.image_url
            },
            "footer": {
              "text": self.footer
            },
            "fields": self.fields.iter().map(
              |d| json!(
                {
                  "name": d.name,
                  "value": d.value,
                  "inline": d.inline
                }
              )
            ).collect::<Vec<Value>>()
          }
        ]
      }
    );

    let token = ENV.bot_token.as_str();

    println!("[INFO] Sending embed...");

    let res = HTTP_CLIENT
      .post(
        format!("{DISCORD_BASE_URL}/channels/{channel_id}/messages")
      )
      .header("Authorization", format!("Bot {token}"))
      .header("Content-Type", "application/json")
      .body(json.to_string())
      .send()?;

    println!("[INFO] Crossposting...");

    let message_id = serde_json::from_str::<Value>(
      &res.text()?
    )?.try_get_string("id")?;

    let res = HTTP_CLIENT
      .post(
        format!("{DISCORD_BASE_URL}/channels/{channel_id}/messages/{message_id}/crosspost")
      )
      .header("Authorization", format!("Bot {token}"))
      .send()?;
    
    if res.status() != StatusCode::OK {
      println!("[WARN] Crosspost failed.")
    }

    Ok(())
  }
}
