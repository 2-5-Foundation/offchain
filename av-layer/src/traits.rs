use jsonrpsee::types::{ErrorObjectOwned, SubscriptionResult};
use jsonrpsee::ws_client::WsClientBuilder;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use primitives::*;
use slab::Slab;

/// Submssion of the transaction object
/// Handling confirmation of transaction from receiver and sender
/// A websocket connection
#[rpc(server, client)]
pub trait Transaction {
    /// Takes in transaction function `call`, `sender address` and `receiver address`
    /// A transaction object will be built based on the params and the object will be subjected for confirmation
    #[method(name = "submit")]
    async fn submit_transaction(
        &self,
        call: Vec<u8>,
        sender: String,
        receiver: String,
    ) -> RpcResult<()>;

    #[method(name = "get_transaction")]
    async fn get_transaction(&self, sender: String, tx_id: Option<Vec<u8>>) -> RpcResult<Vec<u8>>;

    /// Subscription to start listening to any upcoming confirmation request
    /// returns `Vec<TxConfirmationObject>`
    #[method(name = "subscribe")]
    async fn subscribe_confirmation(&self, address: String) -> RpcResult<Option<Vec<TxObject>>>;

    /// Calling this function subscribes
    /// returns `tx_id` for tracking
    #[method(name = "receiverConfirm")]
    async fn receiver_confirmation(
        &self,
        address: String,
        multi_id: String,
        signature: Vec<u8>,
        network: BlockchainNetwork,
    ) -> RpcResult<()>;

    /// Calling this function subscribes
    /// returns `tx_id` for tracking
    #[method(name = "senderConfirm")]
    async fn sender_confirmation(
        &self,
        address: String,
        multi_id: String,
        signature: Vec<u8>,
        network: BlockchainNetwork,
    ) -> RpcResult<()>;
}

// /// Handling confirmation of transaction from receiver and sender
// /// A websocket connection

// /// This should be a websocket connection to network router server
// /// handling propagating to be simulated and
// ///  account control attestation after txn execution ( i.e depositing to the specified acount)
// #[rpc(server, client, namespace = "network_router_feed" )]
// pub trait ToNetworkRouterServer {

// }
