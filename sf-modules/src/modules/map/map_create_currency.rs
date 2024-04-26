use eyre::Result;
use protos::pb::{
  sui::{
    data, move_object_type, object_change, type_tag, CheckpointData, Created, Data, Object, ObjectChange, StructTag, 
    Transaction,
  },
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

fn is_create_coin_metadata_object(tag: &StructTag) -> bool {
  tag.address == "0000000000000000000000000000000000000000000000000000000000000002"
  && tag.module == "coin"
  && tag.name == "CoinMetadata"
}

/// read though the object_changes of all transactions and return the object ids of the objects
/// of CoinMetadata Struct type
fn get_created_coin_metadata(transactions: &Vec<Transaction>) -> Vec<Created> {
  let mut coin_metadata = vec![];

  for tx in transactions {
    for ObjectChange {object_change} in &tx.object_changes {
      if let object_change::ObjectChange::Created(created) = object_change.as_ref().unwrap() {
        if is_create_coin_metadata_object(&created.object_type.as_ref().unwrap()) {
          coin_metadata.push(created.clone())
        }
      };
    }
  };

  coin_metadata
}

/// Get the generic type of the object which essentially defines the coin type
/// This works on onjects like CoinMetadata where the generic type is the Coin type
fn get_metadata_coin_type(obj: &Object) -> String {
  let object_data = obj.data.as_ref().unwrap();
  
  match object_data.data.as_ref().unwrap() {
    data::Data::Move(move_obj) => {
      let struct_tag = move_obj.r#type.as_ref().unwrap();
      
      match struct_tag.move_object_type.as_ref().unwrap() {
        move_object_type::MoveObjectType::Other(type_tag) => {
          // We know for sure that a Coin struct have one sinle generic type i.e. the coin type
          let coin_type_tag = &extract_type_params(type_tag)[0];
          return format!("{}:{}:{}", coin_type_tag.address, coin_type_tag.module, coin_type_tag.name);
        },
        _ => panic!("wrong generic type for CoinMetadata"),
      };
    },
    data::Data::Package(_) => panic!("only move objects"),
  }
}

fn extract_type_params(coin_type_tag: &StructTag) -> Vec<&StructTag> {
  coin_type_tag.type_params.as_ref().unwrap().list.iter().map(|type_tag| {
    if let type_tag::TypeTag::Struct(tag) = type_tag.type_tag.as_ref().unwrap() {
      return tag
    };
    
    panic!("only struct tag");
  }).collect::<Vec<_>>()
}

// The sui coin is created in the Genesis block but not via a function call to the create_currency functions. So
// We have to manually capture it though the object changes
pub fn create_sui_coin(checkpoint_data: CheckpointData) -> Result<Vec<Coin>> {
  let coin_metadata_object = get_created_coin_metadata(&checkpoint_data.transactions);
  let coin_metadata_object_data = checkpoint_data.object_change
  .unwrap()
  .changed_objects
  .into_iter()
  .filter(|c| {
    coin_metadata_object.iter().find(|cm| cm.object_id.eq(&c.object_id)).is_some()
  })
  .collect::<Vec<_>>();

  let mut coins = vec![];

  for coin_created in coin_metadata_object_data {
    // the the coin creator. The account that has sent the tx is the coin creator. Note this does not suggest
    // that the TreasuryCap object i.e. Capability allowing the bearer to mint and burn is owned by the coin creator.
    // In fact we cannot tell who the treasury cap belongs to since it can be wrapped in another object the moment it's
    // created.
    let creator = get_tx_sender(&checkpoint_data.transactions, &coin_created.tx_digest);
    let coin_type = get_metadata_coin_type(&coin_created.object.as_ref().unwrap());

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
      coin_type,
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
