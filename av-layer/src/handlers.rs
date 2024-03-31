use crate::traits::*;
use frame_support::{Blake2_128, StorageHasher};
use futures::stream::{FuturesUnordered, StreamExt};
use jsonrpsee::core::async_trait;
use jsonrpsee::core::{Error::Custom, RpcResult};
use jsonrpsee::types::SubscriptionResult;
use jsonrpsee::SubscriptionSink;
use parity_scale_codec::{Decode, Encode};
use primitives::{BlockchainNetwork, ConfirmationStatus};
use primitives::TxConfirmationObject;
use primitives::TxObject;
use primitives::TxSimulationObject;
use slab::Slab;
use sp_core::ecdsa::{Public as ecdsaPublic, Signature as ECDSASignature};
use sp_core::ed25519::{Public as ed25519Public, Signature as Ed25519Signature};
use sp_core::sr25519::{Public as sr25519Public, Signature as Sr25519Signature};
use sp_core::H256;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::{MultiSignature, MultiSigner};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// A mock database storing each address to the transactions each having a key
/// `address` ===> `multi_id`=====> `Vec<u8>`
pub struct MockDB {
    // Map of multi_id account to array of transaction related to the multi_id
    pub transactions: HashMap<String, Slab<Vec<u8>>>,
    // Map of account id per user ( sender | receiver ) to array of multi_id ( indicating pending transactions)
    pub multi_ids: HashMap<String, Slab<String>>,
    // Map to store confirmation phase of transactions
    // `multi-id` to `TxConfrimationObject`
    pub confirmation: HashMap<String, Vec<u8>>,
}

/// Keeptrack of subscribed clients
pub struct Subscribed {}

// TODO!
// The field should be private
pub struct TransactionHandler {
    pub db: Arc<Mutex<MockDB>>,
}

impl TransactionHandler {
    // Returning transaction data id stored in the database
    pub async fn set_transaction_data(&self, address: String, multi_id: String, data: TxObject) {
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
        let mut inner_db_multi_ids = Slab::<String>::new();

        inner_db_transactions.insert(data.encode());
        inner_db_multi_ids.insert(multi_id.clone());

        db.transactions.insert(address, inner_db_transactions);
        db.multi_ids.insert(multi_id, inner_db_multi_ids);
    }

    pub async fn set_confirmation_transaction_data(
        &self,
        multi_id: String,
        tx_confirmation: TxConfirmationObject,
    ) {
        let mut db = self.db.lock().await;
        db.confirmation.insert(multi_id, tx_confirmation.encode());
    }

    pub async fn get_confirmation_transaction_data(&self, multi_id: String) -> Option<TxConfirmationObject> {
        let db = self.db.lock().await;
        if let Some(confirmation_data) = db.confirmation.get(&multi_id) {
            let tx_confirmation_object: TxConfirmationObject = Decode::decode(&mut &confirmation_data[..]).expect("Failed to decode tx confirmation object");
            Some(tx_confirmation_object)
        }else{
            return None
        }
    }

    pub async fn get_pending_multi_ids(&self, account: String) -> Option<Vec<String>> {
        let db = self.db.lock().await;
        let mut multi_ids_vec = Vec::<String>::new();
        if let Some(multi_ids) = db.multi_ids.get(&account) {
            multi_ids
                .into_iter()
                .for_each(|item| multi_ids_vec.push(item.1.clone()));
            Some(multi_ids_vec)
        } else {
            None
        }
    }

    pub async fn get_transactions(&self, multi_id: String) -> Vec<Vec<u8>> {
        let db = self.db.lock().await;
        let mut tx_vec = Vec::<Vec<u8>>::new();

        let transactions = db.transactions.get(&multi_id).unwrap().clone();
        transactions.into_iter().for_each(|tx| tx_vec.push(tx.1));
        return tx_vec;
    }
}

pub struct ToNetworkRouterHandler {}

async fn process_multi_ids<T, F, Fut>(multi_ids: T, mut f: F)
where
    T: IntoIterator<Item = String>,
    F: FnMut(String) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let mut futures = FuturesUnordered::new();

    for multi_id in multi_ids {
        futures.push(f(multi_id));
    }

    while let Some(_) = futures.next().await {}
}

#[async_trait]
impl TransactionServer for TransactionHandler {
    async fn submit_transaction(
        &self,
        call: Vec<u8>,
        sender: String,
        receiver: String,
    ) -> RpcResult<()> {
        // construct transaction object
        let tx_object = TxObject::new(
            call,
            sender.clone(),
            receiver.clone(),
            primitives::BlockchainNetwork::Polkadot,
        );
        println!("submitting transaction and preparing for confirmation phase");
        // record the tx object to the db
        let multi_id = tx_object.get_multi_id();
        // record for sender
        self.set_transaction_data(sender, multi_id.clone(), tx_object.clone())
            .await;
        // record for receiver
        self.set_transaction_data(receiver, multi_id, tx_object)
            .await;
        Ok(())
    }

    async fn get_transaction(&self, sender: String, tx_id: Option<Vec<u8>>) -> RpcResult<Vec<u8>> {
        Ok(vec![])
    }

    async fn subscribe_confirmation(&self, address: String) -> RpcResult<Option<Vec<TxObject>>> {
        // send all the multi_id pending
        let multi_ids = self.get_pending_multi_ids(address).await;

        let mut tx_object_vec = Vec::<TxObject>::new();

        if let Some(multi_ids) = multi_ids {
            for multi_id in multi_ids.clone() {
                let encoded_txs = self.get_transactions(multi_id).await;
                for encoded_tx in encoded_txs {
                    let tx_object: TxObject = Decode::decode(&mut &encoded_tx[..])
                        .expect("Failed to decode transaction object");
                    tx_object_vec.push(tx_object);
                }
            }
            return Ok(Some(tx_object_vec));
        } else {
            return Ok(None);
        }
    }

    async fn receiver_confirmation(
        &self,
        address: String,
        multi_id: String,
        signature: Vec<u8>,
        network: BlockchainNetwork,
    ) -> RpcResult<()> {
        // verify the signature and the address
        match network {
            BlockchainNetwork::Kusama | BlockchainNetwork::Polkadot => {
                let tx_encoded = self.get_transactions(multi_id.clone()).await;
                // record the confirmation
                let tx: TxObject =
                    Decode::decode(&mut &tx_encoded[0][..]).expect("Failed to decode Tx Object");
                let msg = tx.clone().call;
                let sig = Sr25519Signature::from_slice(&signature)
                    .expect("Failed to convert signature sr25519");

                let account_bytes: [u8; 32] = address
                    .encode()
                    .try_into()
                    .expect("Failed to covert address to bytes");
                let public_account = sr25519Public::from_h256(H256::from(account_bytes));
                if sig.verify(&msg[..], &public_account) {
                    let mut tx_confirmation_object: TxConfirmationObject = tx.into();
                    // update the confirmation status
                    tx_confirmation_object.update_confirmation_status(
                        primitives::ConfirmationStatus::WaitingForSender,
                    );
                    tx_confirmation_object.set_receiver_sig(signature);
                    // store the tx confirmation object
                    self.set_confirmation_transaction_data(multi_id, tx_confirmation_object)
                        .await
                };
                Ok(())
            }
            _ => Err(Custom("Blockchain network not supported".to_string())),
        }
    }

    async fn sender_confirmation(
        &self,
        address: String,
        multi_id: String,
        signature: Vec<u8>,
        network: BlockchainNetwork,
    ) -> RpcResult<()> {
        
        match network {
            BlockchainNetwork::Kusama | BlockchainNetwork::Polkadot => {
                let mut tx = self.get_confirmation_transaction_data(multi_id.clone()).await.ok_or(Custom("Confirmation data unavailable".to_string()))?;
                // check if the if the receiver has confirmed
                if tx.get_confirmation_status() != ConfirmationStatus::WaitingForSender {
                    return Err(Custom("Wait for receiver to confirm".to_string()))
                }
                // verify the signature and the address
                let msg = tx.clone().call;
                let sig = Sr25519Signature::from_slice(&signature)
                    .expect("Failed to convert signature sr25519");

                let account_bytes: [u8; 32] = address
                    .encode()
                    .try_into()
                    .expect("Failed to covert address to bytes");
                let public_account = sr25519Public::from_h256(H256::from(account_bytes));
                if sig.verify(&msg[..], &public_account) {
                    tx.update_confirmation_status(ConfirmationStatus::Ready);
                    let _tx_simulation_object:TxSimulationObject = tx.into();
                    // send to the network router layer
                    // TODO !!!!!!!

                };
                Ok(())
            }
            _ => Err(Custom("Blockchain network not supported".to_string())),
        }

    }
}
