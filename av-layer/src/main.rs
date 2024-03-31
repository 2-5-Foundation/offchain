use anyhow::Ok;
use clap::Parser;
use jsonrpsee::core::{async_trait, client::Subscription};
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::server::{Server, ServerBuilder, SubscriptionSink};
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::ws_client::WsClientBuilder;
use primitives::TxConfirmationObject;
use slab::Slab;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

mod handlers;
mod traits;

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

pub struct ServerProfile {
    name: String,
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //let args = AvLayerServerCli::parse();
    // Initialise the database
    let mut mock_db_transactions = HashMap::new();
    mock_db_transactions.insert(String::new(), Slab::<Vec<u8>>::new());

    let mut mock_db_multi_ids = HashMap::new();
    mock_db_multi_ids.insert(String::new(), Slab::<String>::new());

    let confirmation = HashMap::<String, Vec<u8>>::new();

    let rpc_handler = TransactionHandler {
        db: Arc::new(Mutex::new(MockDB {
            transactions: mock_db_transactions,
            multi_ids: mock_db_multi_ids,
            confirmation,
        })),
    };
    println!("Starting server");

    // Initialize the server
    run_rpc_server(rpc_handler, "127.0.0.1:8000".to_owned()).await?;

    Ok(())
}

// /// Handle client
// async fn
async fn run_rpc_server(
    rpc_handler: TransactionHandler,
    url: String,
) -> anyhow::Result<SocketAddr> {
    let server_builder = ServerBuilder::new();

    let server = server_builder.build(url).await?;

    let addr = server.local_addr()?;
    let handle = server.start(rpc_handler.into_rpc())?;

    // In this example we don't care about doing shutdown so let's it run forever.
    // You may use the `ServerHandle` to shut it down or manage it yourself.

    while !handle.is_stopped() {
        tokio::spawn(handle.clone().stopped());
    }

    Ok(addr)
}
