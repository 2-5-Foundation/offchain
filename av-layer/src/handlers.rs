use std::{collections::HashMap, sync::Arc};
use std::rc::Rc;
use std::cell::RefCell;
use btree_slab::BTreeMap;
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::SubscriptionResult;
use jsonrpsee::SubscriptionSink;
use tokio::sync::Mutex;
use jsonrpsee::core::async_trait;

// use jsonrpsee::core::server::{
//     IntoSubscriptionCloseResponse, PendingSubscriptionSink, SubscriptionCloseResponse, SubscriptionMessage,
// };
use crate::traits::*;

/// A mock database storing each address to the transactions each having a key
/// `address` ===> `tx_id`=====> `Vec<u8>`
pub struct MockDB(pub HashMap<String,BTreeMap<String,Vec<u8>>>);

// TODO!
// The field should be private
pub struct TxSubmissionHandler {
    pub db: Arc<Mutex<MockDB>>
}
pub struct TxConfirmationHandler {
    pub db: Arc<Mutex<MockDB>>
}
pub struct ToNetworkRouterHandler {}


// ======================================================================

#[async_trait]
impl TransactionServer for TxSubmissionHandler {
    async fn submit_transaction(&self, call: Vec<u8>, sender: String, receiver: String) -> RpcResult<()>{
        println!("sender");
        Ok(())
    }

    async fn get_transaction(&self, sender: String, tx_id: Option<Vec<u8>>) -> RpcResult<Vec<u8>>{
        Ok(vec![])
    }
}


impl TransactionConfirmationServer for TxConfirmationHandler {
    fn subscribe_confirmation(&self, sub: SubscriptionSink, address: String) -> SubscriptionResult{
        Ok(())
    }

    fn receiver_confirmation(&self, sub: SubscriptionSink, signature: Vec<u8>)-> SubscriptionResult{
        Ok(())
    }
    
    fn sender_confirmation(&self, sub: SubscriptionSink, signature: Vec<u8>)-> SubscriptionResult{
        Ok(())
    }
}
