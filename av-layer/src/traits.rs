use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use primitives::*;
use jsonrpsee::types::{ErrorObjectOwned, SubscriptionResult};
use jsonrpsee::ws_client::WsClientBuilder;

/// Submssion of the transaction object
#[rpc(server, client, namespace = "transaction_submission" )]
pub trait Transaction {

    /// Takes in transaction function `call`, `sender address` and `receiver address`
    /// A transaction object will be built based on the params and the object will be subjected for confirmation
    #[method(name = "submit")]
    async fn submit_transaction(&self, call: Vec<u8>, sender: String, receiver: String) -> RpcResult<()>;

    #[method(name = "get_transaction")]
    async fn get_transaction(&self, sender: String, tx_id: Option<Vec<u8>>) -> RpcResult<Vec<u8>>;

}

/// Handling confirmation of transaction from receiver and sender
/// A websocket connection
#[rpc(server, client, namespace = "confirmation" )]
pub trait TransactionConfirmation {

    /// Subscription to start listening to any upcoming confirmation request
    /// returns `Vec<TxConfirmationObject>`
    #[subscription(name = "subscribe", item = Vec<TxConfirmationObject>)]
    fn subscribe_confirmation(&self, address: String);

    /// Calling this function subscribes
    /// returns `tx_id` for tracking 
    #[subscription(name = "receiverConfirm", unsubscribe = "receiver_confirm_unsub", item = String )]
    fn receiver_confirmation(&self, signature: Vec<u8>);

    /// Calling this function subscribes
    /// returns `tx_id` for tracking 
    #[subscription(name = "senderConfirm",unsubscribe = "sender_confirm_unsub", item = String)]
    fn sender_confirmation(&self, signature: Vec<u8>);

}

// /// This should be a websocket connection to network router server
// /// handling propagating to be simulated and
// ///  account control attestation after txn execution ( i.e depositing to the specified acount)
// #[rpc(server, client, namespace = "network_router_feed" )]
// pub trait ToNetworkRouterServer {

// }