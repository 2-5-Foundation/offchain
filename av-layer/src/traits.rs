use jsonrpsee::core::SubscriptionResult;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use primitives::*;
use subxt::utils::{AccountId32, MultiAddress, MultiSignature};
/// Submssion of the transaction object
/// Handling confirmation of transaction from receiver and sender
/// A websocket connection
#[rpc(server, client)]
pub trait Transaction {
    /// Takes in transaction function `call`, `sender address` and `receiver address`
    /// A transaction object will be built based on the params and the object will be subjected for confirmation
    #[method(name = "submitTransaction")]
    async fn submit_transaction(
        &self,
        call_data: VaneCallData,
        sender: VaneMultiAddress<AccountId32, ()>,
        receiver: VaneMultiAddress<AccountId32, ()>,
    ) -> RpcResult<()>;

    #[method(name = "getTransaction")]
    async fn get_transaction(
        &self,
        sender: VaneMultiAddress<AccountId32, ()>,
        tx_id: Option<Vec<u8>>,
    ) -> RpcResult<Vec<u8>>;

    /// Subscription to start listening to any upcoming confirmation request
    /// returns `Vec<TxConfirmationObject>` in encoded format
    #[subscription(name = "receiverSubscribeTxConfirmation", unsubscribe= "receiverUnsubscribeTxConfirmation", item=Vec<Vec<u8>>)]
    async fn receiver_subscribe_tx_confirmation(
        &self,
        address: VaneMultiAddress<AccountId32, ()>,
    ) -> SubscriptionResult;

    /// Subscriptiom for sender to listen to incoming confirmed transactions from the receiver
    #[subscription(name = "senderSubscribeTxConfirmation",unsubscribe= "senderUnsubscribeTxConfirmation", item=Vec<TxConfirmationObject>)]
    async fn sender_subscribe_tx_confirmation(
        &self,
        address: VaneMultiAddress<AccountId32, ()>,
    ) -> SubscriptionResult;
    /// Calling this function subscribes
    /// returns `tx_id` for tracking
    #[method(name = "receiverConfirm")]
    async fn receiver_confirmation(
        &self,
        address: VaneMultiAddress<AccountId32, ()>,
        multi_id: VaneMultiAddress<AccountId32, ()>,
        signature: Vec<u8>,
        network: BlockchainNetwork,
    ) -> RpcResult<()>;

    /// Calling this function subscribes
    /// returns `tx_id` for tracking
    #[method(name = "senderConfirm")]
    async fn sender_confirmation(
        &self,
        address: VaneMultiAddress<AccountId32, ()>,
        multi_id: VaneMultiAddress<AccountId32, ()>,
        signature: Vec<u8>,
        network: BlockchainNetwork,
    ) -> RpcResult<()>;

    /// Revert transaction in address verification layer
    #[method(name = "senderRevert")]
    async fn sender_revert_transaction(
        &self,
        address: VaneMultiAddress<AccountId32, ()>,
        multi_id: VaneMultiAddress<AccountId32, ()>,
        network: BlockchainNetwork,
    ) -> RpcResult<()>;

    /// Subscribe to reverted transactions
    #[subscription(name = "subscribeRevertTx",unsubscribe= "unsubscribeRevertTx", item=Vec<TxConfirmationObject>)]
    async fn subscribe_revert_tx(&self) -> SubscriptionResult;

    /// Handling confirmation of transaction from receiver and sender
    /// A websocket connection

    /// This should be a websocket connection to network router server
    /// handling propagating to be simulated and
    /// account control attestation after txn execution ( i.e depositing to the specified acount)
    #[subscription(name = "receiveConfirmedTx", unsubscribe = "unsubReceiveConfirmedTx", item=TxSimulationObject)]
    async fn receive_confirmed_tx(&self) -> SubscriptionResult;
}
