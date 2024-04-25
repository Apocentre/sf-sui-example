use eyre::Result;
use protos::pb::{
  sui::{
    CheckpointData, object_status, object_type, move_object_type, sui_raw_data, type_tag, StructTag,
    CheckpointTransactionBlockResponse, sui_transaction_block_data,
  },
  sui_coin_example::{Coins, Coin},
};
use crate::sui_structs::coin::CoinMetadata;

fn is_create_coin_metadata_object(tag: &StructTag) -> bool {
  tag.address == "0000000000000000000000000000000000000000000000000000000000000002"
  && tag.module == "coin"
  && tag.name == "CoinMetadata"
}

fn get_tx_sender(transaction_block: &[CheckpointTransactionBlockResponse], digest: &str) -> String {
  let tx = transaction_block.iter().find(|tb| tb.digest == digest).unwrap();
  let tx_block_data =  tx.transaction.as_ref().unwrap().data.as_ref().unwrap().sui_transaction_block_data.as_ref().unwrap();
  
  match tx_block_data {
    sui_transaction_block_data::SuiTransactionBlockData::V1(tx_block_data) => tx_block_data.sender.clone(),
  }
}

// The sui coin is created in the Genesis block but not via a function call to the create_currency functions. So
// We have to manually capture it though the object changes
pub fn create_sui_coin(checkpoint_data: CheckpointData) -> Result<Vec<Coin>> {
  let mut coins = vec![];
  let coins_created = checkpoint_data.object_change
  .unwrap()
  .changed_objects
  .iter()
  .filter(|c| c.coin_type.is_some())
  .collect::<Vec<_>>();

  for coin_created in coins_created {
    
  }

  // for object_change in checkpoint_data.object_change.unwrap().changed_objects {
  //   if let object_status::ObjectStatus::Created(()) = object_change.status.unwrap().object_status.unwrap() {
  //     let object_data = object_change.data.unwrap();
  //     let obj_type = object_data.r#type.unwrap().object_type.unwrap();

  //     if let object_type::ObjectType::Struct(stuct_obj) = obj_type {
  //       match stuct_obj.move_object_type.as_ref().unwrap() {
  //         move_object_type::MoveObjectType::Other(object_type) if is_create_coin_metadata_object(object_type) => {
  //           // the the coin creator. The account that has sent the tx is the coin creator. Note this does not suggest
  //           // that the TreasuryCap object i.e. Capability allowing the bearer to mint and burn is owned by the coin creator.
  //           // In fact we cannot tell who the treasury cap belongs to since it can be wrapped in another object the moment it's
  //           // created.
  //           let creator = get_tx_sender( &checkpoint_data.transactions, &object_data.previous_transaction.unwrap());

  //           // Get the move object
  //           let sui_raw_data::SuiRawData::MoveObject(move_obj) = object_data.bcs.unwrap().sui_raw_data.unwrap() else {
  //             panic!("at this point only move object data should exist")
  //           };

  //           // get the generic type which essentially defines the coin type
  //           let struct_tag = move_obj.r#type.unwrap();
  //           // We know for sure that a Coin struct have one sinle generic type i.e. the coin type
  //           let generic_type = &struct_tag.type_params.unwrap().list[0];
  //           let type_tag::TypeTag::Struct(coin_type_tag) = generic_type.type_tag.as_ref().unwrap() else {
  //             panic!("at this point only struct tag")
  //           };

  //           let coin_type = format!("{}:{}:{}", coin_type_tag.address, coin_type_tag.module, coin_type_tag.name);

  //           // decode the BCS object data
  //           let CoinMetadata {
  //             id,
  //             decimals,
  //             name,
  //             symbol,
  //             description,
  //             icon_url,
  //           } = bcs::from_bytes::<CoinMetadata>(&move_obj.bcs_bytes)?;

  //           coins.push(Coin {
  //             id: id.to_string(),
  //             creator,
  //             coin_type,
  //             decimals: decimals as u32,
  //             symbol,
  //             name,
  //             description,
  //             icon_url: icon_url.map(|s| s.to_string()),
  //           });
  //         }
  //         _ => {},
  //       }
  //     }
  //   }
  // }

  Ok(coins)
}

#[substreams::handlers::map]
fn map_create_currency(checkpoint_data: CheckpointData) -> Result<Coins> {
  Ok(Coins {
    list: create_sui_coin(checkpoint_data)?,
  })
}
