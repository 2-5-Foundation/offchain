
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Ok;
use jsonrpsee::core::{async_trait, client::Subscription};
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::server::{SubscriptionSink, Server, ServerBuilder};
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::ws_client::WsClientBuilder;

mod traits;
mod handlers;

use handlers::{TxSubmissionHandler,TxConfirmationHandler};
use tokio::sync::Mutex;
use traits::TransactionServer;
use btree_slab::BTreeMap;
use handlers::MockDB;

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let mut mock_db = HashMap::new();
        mock_db.insert(String::new(), BTreeMap::new());

    let rpc_handler = TxSubmissionHandler {
        db: Arc::new(Mutex::new(MockDB(mock_db)))
    };
    
    tokio::spawn(async{
        run_rpc_server(rpc_handler).await
    });


    Ok(())
}

// /// Handle client
// async fn
async fn run_rpc_server(rpc_handler: TxSubmissionHandler) -> anyhow::Result<SocketAddr> {
    let mut server_builder = ServerBuilder::new();

	let server = server_builder.build("127.0.0.1:2000").await?;

	let addr = server.local_addr()?;
	let handle = server.start(rpc_handler.into_rpc())?;

	// In this example we don't care about doing shutdown so let's it run forever.
	// You may use the `ServerHandle` to shut it down or manage it yourself.
	tokio::spawn(async{
        handle
    });

	Ok(addr)
}

// async fn run_ws_server(rpc_handler: TxConfirmationHandler) -> anyhow::Result<SocketAddr> {
//     let mut server_builder = ServerBuilder::new();

// 	let server = server_builder.build("127.0.0.1:2000").await?;

// 	let addr = server.local_addr()?;
// 	let handle = server.start(rpc_handler.into_rpc())?;

// 	// In this example we don't care about doing shutdown so let's it run forever.
// 	// You may use the `ServerHandle` to shut it down or manage it yourself.
// 	tokio::spawn(handle.stopped());

// 	Ok(addr)
// }