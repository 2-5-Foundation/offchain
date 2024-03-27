use async_trait::async_trait;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use primitives::*;
use anyhow::{Result};

// /// Responsible for handling incoming transaction building request from client/users
// #[async_trait]
// pub trait TxHandler {
//     /// Receive transaction object from user via RPC
//     /// Construct the vane transaction object
//     /// Construct batch call
//     /// send to the temp db message queue
//     async fn process_transaction();
// }

// /// Responsible for tracking and propagating the state of confirmation of the transaction
// #[async_trait]
// pub trait TxConfirmation {
//     /// Notify receiver after subscription to receive pending transaction notification
//     async fn notify_receiver_n_sender();
//     /// Process the transaction object after confirmation at each step ( receiver and sender confirmation)
//     async fn process_confirmation();
//     /// Send to network simulation layer
//     async fn send_to_simulation();
// }


/// Submssion of the transaction object
#[rpc(serve, client, namespace = "transaction_submission" )]
pub trait TransactionServer {

    #[method(name = "submit")]
    async fn submit_transaction(&self, transaction:TxObject) -> Result<()>;

    #[method(name = "get_transaction")]
    async fn get_transaction(&self, id: Vec<u8>) -> Result<Vec<u8>>;

}

/// Handling confirmation of transaction from transaction receiver and sender
///  A websocket connection
#[rpc(serve, client, namespace = "transaction_confirmation" )]
pub trait TransactionConfirmationServer {

    #[method(name = "receiver_confirm")]
    async fn receiver_confirmation(&self, signature: Vec<u8>) -> Result<()>;

    #[method(name = "sender_confirm")]
    async fn sender_confirmation(&self, signature: Vec<u8>) -> Result<()>;

    /// Sending the tx confirmation status to subscribed client
    #[method(name = "confirmation_feed")]
    async fn confirmation_feed(&self);

}

/// This should be a websocket connection to network router server
/// handling propagating to be simulated and
///  account control attestation after txn execution ( i.e depositing to the specified acount)
#[rpc(serve, client, namespace = "network_router_feed" )]
pub trait ToNetworkRouterServer {

}