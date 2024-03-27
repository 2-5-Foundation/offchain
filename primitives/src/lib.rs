use chrono;
use parity_scale_codec::{Decode,Encode};
use anyhow::{Result};

pub use common::*;

pub mod common {
    use super::*;
    /// The transaction object which all operations will be applied upon
    /// `call`: encoded transaction function call 
    /// `network`: network to which the transaction will be submitted to
    /// `lifetime`: maximum period of time in minutes should this transaction be valid on confirmation phase
    /// `multi_id`: The computed address from receiver and sender, this should be kept hidden as it will be used for confirmation
    #[derive(Debug, Encode, Decode, Clone)]
    pub struct TxObject {
        tx_id: String,
        pub call: Vec<u8>,
        pub sender_address: String,
        pub receiver_address: String,
        multi_id: String,
        confirmation_status: ConfirmationStatus,
        pub network: BlockchainNetwork,
        pub lifetime: Option<u8>,
        //submitted_time: 
        pub lifetime_status: LifetimeStatus
    }

    #[derive(Debug, Encode, Decode, Clone)]
    pub enum ConfirmationStatus {
        WaitingForReceiver,
        WaitingForSender,
        Accepted,
        Rejected
    }

    impl From<TxObject> for TxConfirmationObject {
        fn from(value: TxObject) -> Self {
            Self {
                tx_id: value.tx_id,
                call: value.call,
                receiver_sig: None,
                sender_sig: None
            }
        }
    }


    #[derive(Debug, Encode, Decode, Clone)]
    pub struct TxSimulationObject {
        tx_id: String,
        call: Vec<u8>,
        network: BlockchainNetwork
    }

    /// Struct to be sent in the network for confirmation from sender and receiver
    #[derive(Debug, Encode, Decode, Clone)]
    pub struct TxConfirmationObject {
        tx_id: String,
        call: Vec<u8>,
        receiver_sig: Option<Vec<u8>>,
        sender_sig: Option<Vec<u8>>
    }

    impl TxConfirmationObject {
        pub fn verify_receiver(&self) -> Result<bool> {
            // TODO !
            Ok(false)
        }
    }

    #[derive(Debug, Encode, Decode, Clone)]
    pub enum LifetimeStatus{
        Valid,
        Invalid
    }

    /// Supported networks
    #[derive(Debug, Encode, Decode, Clone)]
    pub enum BlockchainNetwork {
        Polkadot,
        Kusama,
        Astar,
        Moonbeam,
        Ethereum,
        Optimism,
        Arbitrum,
        Solana
    }
}