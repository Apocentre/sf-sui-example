use async_stream::try_stream;
use futures::{Stream, StreamExt};
use eyre::{Result, ErrReport};
use log::{info, error};
use tonic::Status;
use std::{
  pin::Pin,
  sync::Arc,
  task::{Context, Poll},
  time::Duration,
};
use tokio::time::sleep;
use tokio_retry::strategy::ExponentialBackoff;
use protos::pb::sf::substreams::{
  rpc::v2::{
    BlockScopedData, Response, response, Request
  },
  v1::Modules,
};
use super::substreams::SubstreamsEndpoint;

pub struct SubstreamsStream {
  stream: Pin<Box<dyn Stream<Item = Result<BlockResponse>> + Send>>,
}

impl SubstreamsStream {
  pub fn new(
    endpoint: Arc<SubstreamsEndpoint>,
    cursor: Option<String>,
    modules: Option<Modules>,
    module_name: String,
    start_block: i64,
    end_block: u64,
  ) -> Self {
    SubstreamsStream {
      stream: Box::pin(stream_blocks(
        endpoint,
        cursor,
        modules,
        module_name,
        start_block,
        end_block,
      )),
    }
  }
}

fn stream_blocks(
  endpoint: Arc<SubstreamsEndpoint>,
  cursor: Option<String>,
  modules: Option<Modules>,
  module_name: String,
  start_block_num: i64,
  stop_block_num: u64,
) -> impl Stream<Item = Result<BlockResponse>> {
  let mut latest_cursor = cursor.unwrap_or_else(|| "".to_string());

  let request = Request {
    start_block_num,
    start_cursor: latest_cursor.clone(),
    stop_block_num,
    modules,
    output_module: module_name,
    ..Default::default()
  };

  // Back off exponentially whenever we encounter a connection error or a stream with bad data
  let mut backoff = ExponentialBackoff::from_millis(500).max_delay(Duration::from_secs(45));

  // This attribute is needed because `try_stream!` seems to break detection of `skip_backoff` assignments
  #[allow(unused_assignments)]
  let mut skip_backoff = false;

  try_stream! {
    loop {
      info!("Blockstreams disconnected, connecting (endpoint {}, start block {}, cursor {})",
        &endpoint,
        start_block_num,
        &latest_cursor
      );

      // We just reconnected, assume that we want to back off on errors
      skip_backoff = false;

      let result = endpoint.clone().substreams(request.clone()).await;

      match result {
        Ok(stream) => {
          info!("Blockstreams connected");

          let mut expected_stream_end = stop_block_num != 0;

          for await response in stream {
            match process_substreams_response(response).await {
              Ok(block_response) => {
                match block_response {
                  None => {},
                  Some(block_scoped_data) => {
                    // Reset backoff because we got a good value from the stream
                    backoff = ExponentialBackoff::from_millis(500).max_delay(Duration::from_secs(45));

                    let cursor = block_scoped_data.cursor.clone();
                    yield BlockResponse::New(block_scoped_data);

                    latest_cursor = cursor;
                  }
                }
              },
              Err(err) => {
                error!("Received error {:#}", err);

                // We have an open connection but there was an error processing the Firehose
                // response. We will reconnect the stream after this; this is the case where
                // we actually _want_ to back off in case we keep running into the same error.
                // An example of this situation is if we get invalid block or transaction data
                // that cannot be decoded properly.

                expected_stream_end = true;
                break;
              }
            }
          }

          if !expected_stream_end {
            info!("Stream blocks complete unexpectedly, expecting stream to always stream blocks");
          } else {
            return
          }
        },
        Err(e) => {
          // We failed to connect and will try again; this is another
          // case where we actually _want_ to back off in case we keep
          // having connection errors.

          error!("Unable to connect to endpoint: {:#}", e);
        }
      }

      // If we reach this point, we must wait a bit before retrying, unless `skip_backoff` is true
      if !skip_backoff {
        if let Some(duration) = backoff.next() {
          sleep(duration).await
        }
      }
    }
  }
}
pub enum BlockResponse {
  New(BlockScopedData),
}

async fn process_substreams_response(
  result: Result<Response, Status>,
) -> Result<Option<BlockScopedData>> {
  let response = match result {
    Ok(v) => v,
    Err(e) => {
      return Err(ErrReport::msg(format!(
        "An error occurred while streaming blocks: {:?}",
        e
      )))
    }
  };
  match response.message {
    Some(response::Message::BlockScopedData(block_scoped_data)) => Ok(Some(block_scoped_data)),
    None => {
      info!("Got None on substream message");
      Ok(None)
    }
    _ => Ok(None),
  }
}

impl Stream for SubstreamsStream {
  type Item = Result<BlockResponse>;

  fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    self.stream.poll_next_unpin(cx)
  }
}
