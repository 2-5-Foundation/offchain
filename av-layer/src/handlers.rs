use crate::traits::*;
use anyhow::ensure;
use jsonrpsee::core::{async_trait, SubscriptionResult};
use jsonrpsee::core::{Error::Custom, RpcResult};
use jsonrpsee::{PendingSubscriptionSink, SubscriptionMessage};
use parity_scale_codec::{Decode, Encode};
use primitives::{
    BlockchainNetwork, ConfirmationStatus, MultiId, TxConfirmationObject, TxObject,
    TxSimulationObject, VaneCallData, VaneMultiAddress,
};
use serde_json::Value as JsonValue;
use sp_core::ecdsa::{Public as ecdsaPublic, Signature as ECDSASignature};
use sp_core::ed25519::{Public as ed25519Public, Signature as Ed25519Signature};
use sp_core::sr25519::{Public as sr25519Public, Signature as Sr25519Signature};
use sp_core::H256;
use sp_runtime::traits::Verify;
use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    sync::Arc,
};
use subxt::utils::{AccountId32, MultiAddress, MultiSignature};
use tokio::sync::Mutex;

/// A mock database storing each address to the transactions each having a key
/// `address` ===> `multi_id`=====> `Vec<u8>`
pub struct MockDB {
    // ============================================================================
    // DB_DATA

    // Map of multi_id account to encoded transactions
    pub transactions: BTreeMap<MultiId, Vec<u8>>,
    // Map of account id per user ( sender | receiver ) to array of multi_id ( indicating pending transactions)
    pub multi_ids: BTreeMap<VaneMultiAddress<AccountId32, ()>, Vec<MultiId>>,
    // Map to store confirmation phase of transactions
    // `multi-id` to `TxConfrimationObject`
    pub confirmation: BTreeMap<MultiId, Vec<u8>>,
    // Store ready to be simulated tx `TxSimulationObject` (queue)
    pub simulation: VecDeque<Vec<u8>>,
    // Record reverted transactions per sender
    pub reverted_transactions: BTreeMap<VaneMultiAddress<AccountId32, ()>, Vec<u8>>,

    // ============================================================================
    // METRICS

    // Keep track of subscibed clients "id"
    pub subscribed: Vec<JsonValue>,
}

// TODO!
// The field should be private
pub struct TransactionHandler {
    pub db: Arc<Mutex<MockDB>>,
}

impl TransactionHandler {
    // DB_DATA

    pub async fn set_transaction_data(
        &self,
        address: VaneMultiAddress<AccountId32, ()>,
        multi_id: MultiId,
        data: TxObject,
    ) {
        let mut db = self.db.lock().await;

        let mut inner_db_multi_ids = Vec::<VaneMultiAddress<AccountId32, ()>>::new();

        inner_db_multi_ids.push(multi_id.clone());

        db.transactions.insert(multi_id, data.encode());
        db.multi_ids.insert(address.clone(), inner_db_multi_ids);
        tracing::info!("recorded tx to the memory db for {:?}", address)
    }

    pub async fn set_confirmation_transaction_data(
        &self,
        multi_id: MultiId,
        tx_confirmation: TxConfirmationObject,
    ) {
        let mut db = self.db.lock().await;
        db.confirmation.insert(multi_id, tx_confirmation.encode());
        tracing::info!("recorded confirmation tx data to the memory db")
    }

    pub async fn get_confirmation_transaction_data(
        &self,
        multi_id: MultiId,
    ) -> Option<TxConfirmationObject> {
        let db = self.db.lock().await;
        if let Some(confirmation_data) = db.confirmation.get(&multi_id) {
            let tx_confirmation_object: TxConfirmationObject =
                Decode::decode(&mut &confirmation_data[..])
                    .expect("Failed to decode tx confirmation object");
            Some(tx_confirmation_object)
        } else {
            return None;
        }
    }

    pub async fn get_pending_multi_ids(
        &self,
        account: VaneMultiAddress<AccountId32, ()>,
    ) -> Option<Vec<MultiId>> {
        let db = self.db.lock().await;
        if let Some(multi_ids) = db.multi_ids.get(&account) {
            Some(multi_ids.to_owned())
        } else {
            None
        }
    }

    pub async fn get_transaction(&self, multi_id: MultiId) -> Option<TxObject> {
        let db = self.db.lock().await;

        if let Some(transaction) = db.transactions.get(&multi_id) {
            let decoded_tx_object: TxObject =
                Decode::decode(&mut &transaction[..]).expect("Failed to decode tx object");
            return Some(decoded_tx_object);
        } else {
            None
        }
    }

    pub async fn get_reverted_txs(&self) -> Vec<TxConfirmationObject> {
        let db = self.db.lock().await;
        let reverted_txs: Vec<TxConfirmationObject> = db
            .reverted_transactions
            .values()
            .into_iter()
            .map(|tx| {
                let decoded_tx: TxConfirmationObject = Decode::decode(&mut &tx[..]).expect("hh");
                decoded_tx
            })
            .collect();
        return reverted_txs;
    }

    pub async fn propagate_tx(&self, tx_simulate: TxSimulationObject) {
        let mut db = self.db.lock().await;
        db.simulation.push_front(tx_simulate.encode());
        tracing::info!("recorded simulation tx object to the memory db")
    }

    pub async fn get_total_number_of_simulated_tx(&self) -> u32 {
        let db = self.db.lock().await;
        db.simulation.len().try_into().unwrap()
    }

    pub async fn get_simulate_tx(&self) -> Option<TxSimulationObject> {
        let mut db = self.db.lock().await;
        if let Some(tx) = db.simulation.pop_front() {
            let tx_sim: TxSimulationObject =
                Decode::decode(&mut &tx[..]).expect("Failed to decode tx simulation object");
            Some(tx_sim)
        } else {
            None
        }
    }

    pub async fn record_reverted_tx(
        &self,
        sender: VaneMultiAddress<AccountId32, ()>,
        reverted_tx: TxConfirmationObject,
    ) {
        let mut db = self.db.lock().await;
        db.reverted_transactions
            .insert(sender, reverted_tx.encode());
    }
    // METRICS

    pub async fn record_subscriber(&self, id: JsonValue) {
        let mut db = self.db.lock().await;
        db.subscribed.push(id);
        db.subscribed.dedup();
    }
}

#[async_trait]
impl TransactionServer for TransactionHandler {
    /// construct tx object and generate the multi id
    /// record the multi_id and set storage for sender and receiver
    /// of the tx data with multi_id being the key
    async fn submit_transaction(
        &self,
        call_data: VaneCallData,
        sender: VaneMultiAddress<AccountId32, ()>,
        receiver: VaneMultiAddress<AccountId32, ()>,
    ) -> RpcResult<()> {
        // construct transaction object
        let tx_object = TxObject::new(
            call_data,
            sender.clone().into(),
            receiver.clone().into(),
            primitives::BlockchainNetwork::Polkadot,
        );
        tracing::info!("submitting transaction and preparing for confirmation phase");
        // record the tx object to the db
        let multi_id = tx_object.get_multi_id();
        // record for sender
        self.set_transaction_data(sender.into(), multi_id.clone(), tx_object.clone())
            .await;
        // record for receiver
        self.set_transaction_data(receiver.into(), multi_id, tx_object)
            .await;
        Ok(())
    }

    async fn get_transaction(
        &self,
        _sender: VaneMultiAddress<AccountId32, ()>,
        _tx_id: Option<Vec<u8>>,
    ) -> RpcResult<Vec<u8>> {
        todo!()
    }

    async fn receiver_subscribe_tx_confirmation(
        &self,
        pending: PendingSubscriptionSink,
        address: VaneMultiAddress<AccountId32, ()>,
    ) -> SubscriptionResult {
        let sink = pending.accept().await?;
        let sub_id: JsonValue = sink.subscription_id().into();
        // record metrics
        self.record_subscriber(sub_id).await;
        // send all the multi_id pending
        let multi_ids = self.get_pending_multi_ids(address.into()).await;

        let mut txs_vec = Vec::<TxObject>::new();
        if let Some(multi_ids) = multi_ids {
            for multi_id in multi_ids.clone() {
                let encoded_txs = self.get_transaction(multi_id).await;
                if let Some(tx) = encoded_txs {
                    txs_vec.push(tx)
                }
            }

            sink.send(SubscriptionMessage::from_json(&txs_vec)?).await?;
        } else {
            let empty_result = Vec::<Vec<u8>>::new();
            sink.send(SubscriptionMessage::from_json(&empty_result)?)
                .await?;
        }
        tracing::info!("subcribed to tx confirmation receiver");
        Ok(())
    }

    // Subscribe for sender to listen to confirmed tx from the receiver
    async fn sender_subscribe_tx_confirmation(
        &self,
        pending: PendingSubscriptionSink,
        address: VaneMultiAddress<AccountId32, ()>,
    ) -> SubscriptionResult {
        let sink = pending.accept().await?;
        let sub_id: JsonValue = sink.subscription_id().into();
        // record metrics
        self.record_subscriber(sub_id).await;

        let multi_ids = self.get_pending_multi_ids(address.into()).await;

        let mut txs_vec = Vec::<TxConfirmationObject>::new();
        if let Some(multi_ids) = multi_ids {
            for multi_id in multi_ids.clone() {
                let encoded_txs = self.get_confirmation_transaction_data(multi_id).await;
                if let Some(tx) = encoded_txs {
                    txs_vec.push(tx)
                }
            }

            sink.send(SubscriptionMessage::from_json(&txs_vec)?).await?;
        } else {
            let empty_result = Vec::<Vec<u8>>::new();
            sink.send(SubscriptionMessage::from_json(&empty_result)?)
                .await?;
        }
        tracing::info!("subcribed to tx confirmation sender");
        Ok(())
    }

    async fn receiver_confirmation(
        &self,
        address: VaneMultiAddress<AccountId32, ()>,
        multi_id: VaneMultiAddress<AccountId32, ()>,
        signature: Vec<u8>,
        network: BlockchainNetwork,
    ) -> RpcResult<()> {
        // verify the signature and the address
        match network {
            BlockchainNetwork::Kusama | BlockchainNetwork::Polkadot => {
                let tx = self
                    .get_transaction(multi_id.clone().into())
                    .await
                    .ok_or(Custom("Transaction Not Found".to_string()))?;
                // record the confirmation

                // message to sign for address verification
                let msg = tx.clone().get_tx_id();

                let sig = Sr25519Signature::from_slice(&signature)
                    .ok_or(Custom("Failed to convert signature sr25519".to_string()))?;

                let account_bytes: [u8; 32] = address
                    .encode()
                    .try_into()
                    .expect("Failed to covert address to bytes");
                let public_account = sr25519Public::from_h256(H256::from(account_bytes));

                if sig.verify(&msg.encode()[..], &public_account) {
                    let mut tx_confirmation_object: TxConfirmationObject = tx.into();

                    // update the confirmed address
                    tx_confirmation_object.set_confirmed_receiver(address);

                    // update the confirmation status
                    tx_confirmation_object.update_confirmation_status(
                        primitives::ConfirmationStatus::WaitingForSender,
                    );
                    // store the tx confirmation object
                    self.set_confirmation_transaction_data(multi_id.into(), tx_confirmation_object)
                        .await
                };
                tracing::info!("receiver confirmed");
                Ok(())
            }
            _ => Err(Custom("Blockchain network not supported".to_string())),
        }
    }

    async fn sender_confirmation(
        &self,
        address: VaneMultiAddress<AccountId32, ()>,
        multi_id: VaneMultiAddress<AccountId32, ()>,
        signature: Vec<u8>,
        network: BlockchainNetwork,
    ) -> RpcResult<()> {
        match network {
            BlockchainNetwork::Kusama | BlockchainNetwork::Polkadot => {
                let mut tx = self
                    .get_confirmation_transaction_data(multi_id.clone().into())
                    .await
                    .ok_or(Custom("Confirmation data unavailable".to_string()))?;

                // check if the if the receiver has confirmed
                if tx.get_confirmation_status() != ConfirmationStatus::WaitingForSender {
                    return Err(Custom("Wait for receiver to confirm".to_string()));
                }
                // verify the signature and the address

                // message to sign for address verification
                let msg = tx.clone().get_tx_id();

                let sig = Sr25519Signature::from_slice(&signature)
                    .expect("Failed to convert signature sr25519");

                let account_bytes: [u8; 32] = address
                    .encode()
                    .try_into()
                    .expect("Failed to covert address to bytes");
                let public_account = sr25519Public::from_h256(H256::from(account_bytes));

                if sig.verify(&msg.encode()[..], &public_account) {
                    // confirm the resulting multi_id
                    let multi_id = tx.calculate_confirmed_multi_id(address.clone());

                    if multi_id == tx.get_multi_id() {
                        tx.set_confirmed_sender(address);

                        tx.update_confirmation_status(ConfirmationStatus::Ready);
                        let tx_simulation_object: TxSimulationObject = tx.into();
                        // store to the ready to be simulated tx storage
                        self.propagate_tx(tx_simulation_object).await;
                        tracing::info!("sender confirmed");
                    } else {
                        // record to reverted tx, as this tx is automatically reverted due to mismatch in address confirmation
                        tx.update_confirmation_status(ConfirmationStatus::RejectedMismatchAddress);

                        tracing::info!("tx reverted due to mismatched confirmed address");
                        self.record_reverted_tx(address, tx).await;
                        Err(Custom("Address Mismatched , Tx reverted".to_string()))?;
                    }
                };
                Ok(())
            }
            _ => Err(Custom("Blockchain network not supported".to_string())),
        }
    }

    async fn sender_revert_transaction(
        &self,
        address: VaneMultiAddress<AccountId32, ()>,
        multi_id: VaneMultiAddress<AccountId32, ()>,
        network: BlockchainNetwork,
    ) -> RpcResult<()> {
        todo!()
    }

    async fn subscribe_revert_tx(&self, pending: PendingSubscriptionSink) -> SubscriptionResult {
        let sink = pending.accept().await?;
        let reverted_txs: Vec<TxConfirmationObject> = self.get_reverted_txs().await;
        let json_reverted_txs = SubscriptionMessage::from_json(&reverted_txs)?;
        sink.send(json_reverted_txs).await?;
        Ok(())
    }

    async fn receive_confirmed_tx(&self, pending: PendingSubscriptionSink) -> SubscriptionResult {
        let sink = pending.accept().await?;
        // fetch the confirmed and ready to be simulated txn
        while self.get_total_number_of_simulated_tx().await != 0 {
            if let Some(tx_simulated) = self.get_simulate_tx().await {
                sink.send(SubscriptionMessage::from_json(&tx_simulated)?)
                    .await?;
            }
        }
        Ok(())
    }
}
