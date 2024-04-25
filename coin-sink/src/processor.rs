use std::{sync::Arc, fs};
use eyre::Result;
use log::{info};
use futures::StreamExt;
use protos::pb::{
  sf::substreams::{v1::{Package, Module}, rpc::v2::MapModuleOutput},
  sui_coin_example::Coins
};
use prost::Message;
use crate::{
  config::Config,
  core::{
    substreams::SubstreamsEndpoint,
    stream::{BlockResponse, SubstreamsStream},
  },
};

pub struct Processor {
  stream: SubstreamsStream,
  ident_checks_done: bool,
}

impl Processor {
  pub async fn new(config: &Config, start_block: i64) -> Result<Self> {
    Ok(Self {
      stream: Self::create_stream(config, start_block).await?,
      ident_checks_done: false,
    })
  }

  pub async fn create_stream(config: &Config, start_block: i64) -> Result<SubstreamsStream> {
    let endpoint = Arc::new(
      SubstreamsEndpoint::new(
        &config.firehose_endpoint,
        Some("".to_string()),
      )
      .await?,
    );
    let package = Self::read_package(&config.package_file, start_block as u64)?;

    let stream = SubstreamsStream::new(
      endpoint.clone(),
      None,
      package.modules,
      config.module_name.clone(),
      start_block,
      config.stop_block,
    );

    Ok(stream)
  }

  pub async fn run(&mut self) -> Result<()> {
    info!("Processor started");

    while let Some(event) = self.stream.next().await {
      match event {
        Ok(BlockResponse::New(data)) => {
          let block = data.clock.as_ref().unwrap().number;
          let outputs = data.output.into_iter()
          .filter(|o| o.name.eq("map_create_currency"))
          .collect::<Vec<MapModuleOutput>>();

          for output in outputs {
            println!("{:?}", output);
            
            if let Some(coins) = unwrap_data(output) {
              for coin in coins.list {
                println!("Coin is {:?}", coin);
              }
            }
          }

          info!("Checkpoint Processed {}", block);
          self.ident_checks_done = true;
        }
        Err(error) => {
          panic!("Error from stream {:?}", error);
        }
      }
    }

    info!("Processor stopped");
    Ok(())
  }

  fn read_package(file: &str, start_block: u64) -> Result<Package> {
    let content = fs::read(file)?;
    let mut package = Package::decode(content.as_ref())?;
    let mut modules = package.modules.clone().unwrap();
    let inner_modules = modules
    .modules
    .clone()
    .into_iter()
    .map(|mut m| {
      m.initial_block = start_block;
      m
    })
    .collect::<Vec<Module>>();

    modules.modules = inner_modules;
    package.modules = Some(modules);

    Ok(package)
  }
}

pub fn unwrap_data(output: MapModuleOutput) -> Option<Coins> {
  if let Some(map_output) = &output.map_output {
    if map_output.value.len() > 0 {
      let coins: Coins = decode(&map_output.value).unwrap();

      if coins.list.len() > 0 {
        return Some(coins);
      }
    }
  }

  None
}

pub fn decode<T: Default + prost::Message>(buf: &Vec<u8>) -> Result<T> {
  Ok(prost::Message::decode(&buf[..])?)
}

