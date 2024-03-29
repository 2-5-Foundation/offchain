use btree_slab::BTreeMap;
use jsonrpsee::core::async_trait;
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::SubscriptionResult;
use jsonrpsee::SubscriptionSink;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use crate::traits::*;

/// A mock database storing each address to the transactions each having a key
/// `address` ===> `tx_id`=====> `Vec<u8>`
pub struct MockDB(pub HashMap<String, BTreeMap<String, Vec<u8>>>);

// TODO!
// The field should be private
pub struct TransactionHandler {
    pub db: Arc<Mutex<MockDB>>,
}

pub struct ToNetworkRouterHandler {}


#[async_trait]
impl TransactionServer for TransactionHandler {
    async fn submit_transaction(
        &self,
        call: Vec<u8>,
        sender: String,
        receiver: String,
    ) -> RpcResult<()> {
        println!("sender");
        Ok(())
    }

    async fn get_transaction(&self, sender: String, tx_id: Option<Vec<u8>>) -> RpcResult<Vec<u8>> {
        Ok(vec![])
    }

    fn subscribe_confirmation(&self, sub: SubscriptionSink, address: String) -> SubscriptionResult {
        Ok(())
    }

    fn receiver_confirmation(
        &self,
        sub: SubscriptionSink,
        signature: Vec<u8>,
    ) -> SubscriptionResult {
        Ok(())
    }

    fn sender_confirmation(&self, sub: SubscriptionSink, signature: Vec<u8>) -> SubscriptionResult {
        Ok(())
    }
}
