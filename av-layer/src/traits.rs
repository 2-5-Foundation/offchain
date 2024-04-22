use jsonrpsee::core::SubscriptionResult;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use primitives::*;

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
        sender: VaneMultiAddress<u128, ()>,
        receiver: VaneMultiAddress<u128, ()>,
    ) -> RpcResult<()>;

    #[method(name = "get_transaction")]
    async fn get_transaction(
        &self,
        sender: VaneMultiAddress<u128, ()>,
        tx_id: Option<Vec<u8>>,
    ) -> RpcResult<Vec<u8>>;

    /// Subscription to start listening to any upcoming confirmation request
    /// returns `Vec<TxConfirmationObject>` in encoded format
    #[subscription(name = "subscribeTxConfirmation", item=Vec<Vec<u8>>)]
    async fn subscribe_tx_confirmation(
        &self,
        address: VaneMultiAddress<u128, ()>,
    ) -> SubscriptionResult;

    /// Subscriptiom for sender to listen to incoming confirmed transactions from the receiver
    #[subscription(name = "subscribeTxConfirmationSender", item=Vec<TxConfirmationObject>)]
    async fn subscribe_tx_confirmation_sender(
        &self,
        address: VaneMultiAddress<u128, ()>,
    ) -> SubscriptionResult;
    /// Calling this function subscribes
    /// returns `tx_id` for tracking
    #[method(name = "receiverConfirm")]
    async fn receiver_confirmation(
        &self,
        address: VaneMultiAddress<u128, ()>,
        multi_id: VaneMultiAddress<u128, ()>,
        signature: Vec<u8>,
        network: BlockchainNetwork,
    ) -> RpcResult<()>;

    /// Calling this function subscribes
    /// returns `tx_id` for tracking
    #[method(name = "senderConfirm")]
    async fn sender_confirmation(
        &self,
        address: VaneMultiAddress<u128, ()>,
        multi_id: VaneMultiAddress<u128, ()>,
        signature: Vec<u8>,
        network: BlockchainNetwork,
    ) -> RpcResult<()>;

    /// Handling confirmation of transaction from receiver and sender
    /// A websocket connection

    /// This should be a websocket connection to network router server
    /// handling propagating to be simulated and
    /// account control attestation after txn execution ( i.e depositing to the specified acount)
    #[subscription(name = "receiveConfirmedTx", unsubscribe = "unsubReceiveConfirmedTx", item=Vec<TxSimulationObject>)]
    async fn receive_confirmed_tx(&self) -> SubscriptionResult;
}
