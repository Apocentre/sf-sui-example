use std::{panic, process, env};
use dotenv;
use env_logger::Env;
use envconfig::Envconfig;
use eyre::Result;
use coin_sink::{config::Config, processor::Processor};

#[tokio::main]
async fn main() -> Result<()> {
  let orig_hook = panic::take_hook();
  panic::set_hook(Box::new(move |panic_info| {
    orig_hook(panic_info);
    process::exit(1);
  }));

  if env::var("ENV").unwrap() == "development" {
    dotenv::from_filename(".env").expect("cannot load env from a file");
  }

  env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
  let config = Box::leak(Box::new(Config::init_from_env().unwrap()));

  let mut processor = Processor::new(&config, config.start_block).await.unwrap();
  processor.run().await?;

  Ok(())
}
