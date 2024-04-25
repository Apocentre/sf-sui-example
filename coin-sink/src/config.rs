use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct Config {
  #[envconfig(from = "FIREHOSE_ENDPOINT")]
  pub firehose_endpoint: String,
  #[envconfig(from = "MODULE_NAME")]
  pub module_name: String,
  #[envconfig(from = "PACKAGE_FILE")]
  pub package_file: String,
  #[envconfig(from = "START_BLOCK")]
  pub start_block: i64,
  #[envconfig(from = "STOP_BLOCK")]
  pub stop_block: u64,
}
