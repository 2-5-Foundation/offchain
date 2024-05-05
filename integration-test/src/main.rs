//! VANE OFFCHAIN INTEGRATION TESTS
//!
//!
//! Testing the following functionalities and stress testing the system overall
//!
//! 1. Submitting transaction
//! 2. Submitting wrong constructed transaction
//! 3. Getting transaction details in the message queue database
//! 4. Client ( Reciver & Sender ) subscribing to receive updates on the ongoing transaction
//! 5. Handling receiver confirmation
//! 6. Error if Sender confirms first
//! 7. Sender confirmation should change the state of the transaction to accepted and ready to be propagated to network router layer
//!

use jsonrpsee::core::client::{ClientT, SubscriptionClientT};
use jsonrpsee::ws_client::{WsClient, WsClientBuilder};
use primitives::{
    BlockchainNetwork, TxConfirmationObject, TxObject, VaneCallData, VaneMultiAddress,
};
use sp_core::ecdsa::{Public as ecdsaPublic, Signature as ECDSASignature};
use sp_core::ed25519::{Public as ed25519Public, Signature as Ed25519Signature};
use sp_core::sr25519::{
    Pair as sr25519Pair, Public as sr25519Public, Signature as Sr25519Signature,
};
use sp_core::{Pair, H256};
use sp_runtime::traits::{IdentifyAccount, Verify};
use subxt::tx::TxPayload;
use subxt::utils::{AccountId32, MultiAddress};
use subxt::{Metadata, OnlineClient, PolkadotConfig};
use subxt_signer::sr25519::{dev, Keypair};
#[subxt::subxt(runtime_metadata_path = "polkadot.scale")]
pub mod polkadot {}

// Polkadot testing
// Solana testing
// Ethereum testing

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // generate accounts
    // let alicePair = sr25519Pair::from_string("//Alice", None).expect("Failed to generate key pair");
    // let bobPair = sr25519Pair::from_string("//Bob", None).expect("Failed to generate key pair");
    let alice = dev::alice().public_key();
    //let bob:VaneMultiAddress<u128,u32> = VaneMultiAddress::Address32(dev::bob().public_key().into());
    // construct a transfer tx
    //let transfer_call = polkadot::tx().balances().transfer_keep_alive(bob, 10_000);
    // send to vane av-layer
    // let av_client =
    // listen to updates
    //let receiver_tx_updates = av_client.request("subscribe",vec![bob]);
    //let sender_tx_updates = av_client.subscribe(subscribe_method, params, unsubscribe_method)
    // receiver confirmation

    // fetch the tx and confirm the state

    // sender confirmation

    // propagating the confirmed tx to network-simulation and routing layer
    Ok(())
}

pub struct PolkadotTest {
    pub client: WsClient,
}

impl PolkadotTest {
    pub async fn connect() -> PolkadotTest {
        let client = WsClientBuilder::default()
            .build("127.0.0.1:8000")
            .await
            .expect("Failed to initilise Ws");

        Self { client }
    }

    pub async fn send_transaction(
        &self,
        sender: Keypair,
        receiver: Keypair,
        amount: u128,
    ) -> anyhow::Result<()> {
        // build a transfer keep alive polkadot call
        let sender_multi: VaneMultiAddress<AccountId32, ()> =
            VaneMultiAddress::Address32(sender.public_key().0);
        let receiver_multi: VaneMultiAddress<AccountId32, ()> =
            VaneMultiAddress::Address32(receiver.public_key().0);

        let vane_call_data = VaneCallData::new(BlockchainNetwork::Polkadot, amount);
        // use the client to submit the transaction to av layer
        if self.client.is_connected() {
            self.client
                .request(
                    "submitTransaction",
                    vec![sender_multi,receiver_multi],
                )
                .await?;
        }
        Ok(())
    }

    pub async fn listen_to_incoming_tx(address: MultiAddress<u128, ()>) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn sender_listen_confirmed_tx(sender: MultiAddress<u128, ()>) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn receiver_tx_confirm(receiver_pair: Keypair) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn sender_tx_confirm(sender_pair: Keypair) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn receiver_confirmed_account_ownership() -> anyhow::Result<()> {
        todo!()
    }
}

pub struct SolanaTest;

pub struct EthereumTest;
