use crate::traits::*;
use btree_slab::BTreeMap;
use jsonrpsee::core::async_trait;
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::SubscriptionResult;
use jsonrpsee::SubscriptionSink;
use parity_scale_codec::{Decode, Encode};
use primitives::TxObject;
use slab::Slab;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
/// A mock database storing each address to the transactions each having a key
/// `address` ===> `multi_id`=====> `Vec<u8>`
pub struct MockDB {
    // Map of multi_id account to array of transaction related to the multi_id
    pub transactions: HashMap<String, Slab<Vec<u8>>>,
    // Map of account id per user ( sender | receiver ) to array of multi_id ( indicating pending transactions)
    pub multi_ids: HashMap<String, Slab<String>>
}

// TODO!
// The field should be private
pub struct TransactionHandler {
    pub db: Arc<Mutex<MockDB>>,
}

impl TransactionHandler {
    // Returning transaction data id stored in the database
    pub async fn set_transaction_data(&mut self, address: String,multi_id: String, data: TxObject){
        let mut db = self.db.lock().await;
        if db.transactions.contains_key(&multi_id) {
            let db = db.transactions.get_mut(&multi_id).expect(&format!(
                "Cannot find transaction data with the key {}",
                &address
            ));
            db.insert(data.encode());
            // No need to add multi to the account storage as the key was present
        }

        let mut inner_db_transactions = Slab::<Vec<u8>>::new();
        let mut inner_db_multi_ids= Slab::<String>::new();

        inner_db_transactions.insert(data.encode());
        inner_db_multi_ids.insert(multi_id.clone());
        
        db.transactions.insert(address, inner_db_transactions);
        db.multi_ids.insert(multi_id, inner_db_multi_ids);
    }

    pub async fn get_pending_multi_ids(&self, account:String) -> Option<Slab<String>> {
        let db = self.db.lock().await;
        if let Some(multi_ids) = db.multi_ids.get(&account){
            Some(multi_ids.clone())
        }else{
            None
        }
        
    }

    pub async fn get_transactions(&self, multi_id: String) -> Slab<Vec<u8>> {
        let db = self.db.lock().await;
        let transactions = db.transactions.get(&multi_id).unwrap().clone();
        transactions
    }
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
