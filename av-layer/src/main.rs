use anyhow::Ok;
use clap::Parser;
use jsonrpsee::server::ServerBuilder;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;

mod handlers;
mod traits;

use handlers::MockDB;
use handlers::TransactionHandler;
use parity_scale_codec::alloc::sync::Once;
use tokio::sync::Mutex;
use tracing_subscriber;
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

// Tracing setup
static INIT: Once = Once::new();
pub fn init_tracing() -> anyhow::Result<()> {
    // Add test tracing (from sp_tracing::init_for_tests()) but filtering for xcm logs only
    let vane_subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .finish();

    tracing::dispatcher::set_global_default(vane_subscriber.into())
        .expect("Failed to initialise tracer");
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing()?;
    tracing::info!("initiliasing av layer 🔥⚡️");
    //let args = AvLayerServerCli::parse();
    // Initialise the database
    let mock_db_transactions = BTreeMap::new();
    let mock_db_multi_ids = BTreeMap::new();

    let confirmation = BTreeMap::new();

    let rpc_handler = TransactionHandler {
        db: Arc::new(Mutex::new(MockDB {
            transactions: mock_db_transactions,
            multi_ids: mock_db_multi_ids,
            confirmation,
            reverted_transactions: BTreeMap::new(),
            simulation: VecDeque::new(),
            subscribed: Vec::new(),
        })),
    };

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
