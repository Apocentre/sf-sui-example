use eyre::Result;
use protos::pb::{
  sui::{CheckpointData, Data, Transaction, data},
  sui_coin_example::{Coin, Coins},
};
use crate::sui_structs::coin::CoinMetadata;

fn get_tx_sender(transactions: &[Transaction], digest: &str) -> String {
  let tx = transactions.iter().find(|tb| tb.digest == digest).unwrap();
  let tx_block_data = tx.sender_signed_data.first().unwrap()
  .intent_message.as_ref().unwrap()
  .value.as_ref().unwrap()
  .tx_data.as_ref().unwrap();
  
  match tx_block_data {
    protos::pb::sui::transaction_data::TxData::V1(v1) => v1.sender.clone()
  }
}

fn get_object_data(source: Data) -> Vec<u8> {
  match source.data.unwrap() {
    data::Data::Move(source) => source.contents.clone(),
    data::Data::Package(_) => panic!("must be a coin type object change thus as move call"),
  }
}

// The sui coin is created in the Genesis block but not via a function call to the create_currency functions. So
// We have to manually capture it though the object changes
pub fn create_sui_coin(checkpoint_data: CheckpointData) -> Result<Vec<Coin>> {
  let mut coins = vec![];
  let coins_created = checkpoint_data.object_change
  .unwrap()
  .changed_objects
  .into_iter()
  .filter(|c| c.coin_type.is_some())
  .collect::<Vec<_>>();

  for coin_created in coins_created {
    let creator = get_tx_sender(&checkpoint_data.transactions, &coin_created.tx_digest);

    // decode the BCS object data
    let CoinMetadata {
      id,
      decimals,
      name,
      symbol,
      description,
      icon_url,
    } = bcs::from_bytes::<CoinMetadata>(&get_object_data(coin_created.object.unwrap().data.unwrap()))?;

    coins.push(Coin {
      id: id.to_string(),
      creator,
      coin_type: coin_created.coin_type.unwrap().clone(),
      decimals: decimals as u32,
      symbol,
      name,
      description,
      icon_url: icon_url.map(|s| s.to_string()),
    });
  }

  Ok(coins)
}

#[substreams::handlers::map]
fn map_create_currency(checkpoint_data: CheckpointData) -> Result<Coins> {
  Ok(Coins {
    list: create_sui_coin(checkpoint_data)?,
  })
}
