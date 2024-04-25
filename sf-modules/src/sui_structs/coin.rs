use serde::Deserialize;
use super::core_sui::{ObjectID, Url};

#[derive(Deserialize)]
pub struct CoinMetadata {
  pub id: ObjectID,
  pub decimals: u8,
  pub name: String,
  pub symbol: String,
  pub description: String,
  pub icon_url: Option<Url>,
}
