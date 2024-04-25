use std::str;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AccountAddress([u8; 32]);

#[derive(Deserialize)]
pub struct ObjectID(AccountAddress);

impl ToString for ObjectID {
  fn to_string(&self) -> String {
    hex::encode(&self.0.0)
  }
}

#[derive(Deserialize)]
pub struct Url {
  pub url: String,
}

impl ToString for Url {
  fn to_string(&self) -> String {
    self.url.clone()
  }
}
