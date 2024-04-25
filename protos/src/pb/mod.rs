// Exlude from wasm target
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[cfg(feature = "stream")]
pub mod sf {
  pub mod substreams {
    pub mod rpc {
      pub mod v2{
        include!("sf.substreams.rpc.v2.rs");
      }
    }

    pub mod v1 {
      include!("sf.substreams.v1.rs");
    }
  }
}

pub mod sui {
  include!("sui.checkpoint.v1.rs");
}
pub mod sui_coin_example;
