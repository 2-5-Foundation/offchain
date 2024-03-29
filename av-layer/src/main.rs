use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use clap::Parser;
use anyhow::Ok;
use jsonrpsee::core::{async_trait, client::Subscription};
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::server::{Server, ServerBuilder, SubscriptionSink};
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::ws_client::WsClientBuilder;

mod handlers;
mod traits;

use btree_slab::BTreeMap;
use handlers::MockDB;
use handlers::TransactionHandler;
use tokio::sync::Mutex;
use traits::TransactionServer;

/// Address Verification layer cli server arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct AvLayerServerCli {
    /// Name of the server
    #[arg(short, long)]
    name: String,
    /// url to listen to
    #[arg(short, long)]
    url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let args = AvLayerServerCli::parse();
    // Initialise the database
    let mut mock_db = HashMap::new();
    mock_db.insert(String::new(), BTreeMap::new());

    let rpc_handler = TransactionHandler {
        db: Arc::new(Mutex::new(MockDB(mock_db))),
    };
    // Initialize the server
    tokio::spawn(async { run_rpc_server(rpc_handler, args.url).await });

    Ok(())
}

// /// Handle client
// async fn
async fn run_rpc_server(rpc_handler: TransactionHandler, url: String) -> anyhow::Result<SocketAddr> {
    let server_builder = ServerBuilder::new();

    let server = server_builder.build(url).await?;

    let addr = server.local_addr()?;
    let handle = server.start(rpc_handler.into_rpc())?;

    // In this example we don't care about doing shutdown so let's it run forever.
    // You may use the `ServerHandle` to shut it down or manage it yourself.
    tokio::spawn(async { handle });

    Ok(addr)
}
