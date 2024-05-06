pub use common::*;
use derivative::Derivative;
use frame_support::StorageHasher;
use frame_support::Twox64Concat;
use hex::ToHex;
use hex_literal::hex;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use tinyrand::{Rand, RandRange, StdRand};
pub mod common {

    use std::borrow::Cow;

    use sp_core::{blake2_128, blake2_256};
    use subxt::{
        tx::{DynamicPayload, Payload},
        utils::{AccountId32, MultiAddress, MultiSignature},
    };

    use super::*;
    /// The transaction object which all operations will be applied upon
    /// `call`: encoded transaction function call
    /// `network`: network to which the transaction will be submitted to
    /// `lifetime`: maximum period of time in minutes should this transaction be valid on confirmation phase
    /// `multi_id`: The computed address from receiver and sender, this should be kept hidden as it will be used for confirmation
    #[derive(Debug, Encode, Serialize, Deserialize, Decode, Clone)]
    pub struct TxObject {
        tx_id: String,
        pub call: VaneCallData,
        pub sender_address: VaneMultiAddress<AccountId32, ()>,
        pub receiver_address: VaneMultiAddress<AccountId32, ()>,
        multi_id: VaneMultiAddress<AccountId32, ()>,
        pub network: BlockchainNetwork,
        pub lifetime: Option<u8>,
        //submitted_time:
        pub lifetime_status: LifetimeStatus,
    }

    /// Types for easier code navigation
    pub type MultiId = VaneMultiAddress<AccountId32, ()>;

    impl TxObject {
        pub fn new(
            call: VaneCallData,
            sender_address: MultiAddress<AccountId32, ()>,
            receiver_address: MultiAddress<AccountId32, ()>,
            network: BlockchainNetwork,
        ) -> Self {
            let tx_id_vec = call.get_tx_id();
            let tx_id_hash = Twox64Concat::hash(&tx_id_vec[..]);
            let tx_id = hex::encode(tx_id_hash);

            let multi_id = (sender_address.clone(), receiver_address.clone(), b"VANE")
                .using_encoded(blake2_256);
            let multi_id: VaneMultiAddress<AccountId32, ()> = VaneMultiAddress::Address32(multi_id);
            Self {
                tx_id,
                call,
                sender_address: sender_address.clone().into(),
                receiver_address: sender_address.into(),
                multi_id: multi_id.into(),
                network,
                lifetime: None,
                lifetime_status: LifetimeStatus::Valid,
            }
        }

        pub fn get_multi_id(&self) -> VaneMultiAddress<AccountId32, ()> {
            self.multi_id.clone().into()
        }

        pub fn get_tx_id(self) -> String {
            self.tx_id
        }
    }

    /// VaneCallData represents enumeration on different network transaction function types ( Call )
    /// For Solana calls_data it supports adding accounts id that will receive tokens
    /// The structure is NetworkNameCallData(amaount)
    #[derive(Debug, Encode, Decode, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum VaneCallData {
        SubstrateCallData {
            amount: u128,
        },
        SolanaCallData {
            amount: u128,
            extra_receivers: Vec<VaneMultiAddress<AccountId32, ()>>,
        },
        EthereumCallData {
            amount: u128,
        },
    }

    impl VaneCallData {
        pub fn new(network: BlockchainNetwork, amount: u128) -> Self {
            match network {
                BlockchainNetwork::Polkadot => VaneCallData::SubstrateCallData { amount },
                _ => todo!(),
            }
        }

        pub fn get_tx_id(&self) -> Vec<u8> {
            match self {
                VaneCallData::SubstrateCallData { amount } => {
                    let mut rand = StdRand::default();
                    Twox64Concat::hash(&format!("{}{}", amount, rand.next_u128()).encode()[..])
                }
                _ => todo!(),
            }
        }
    }

    // #[derive(Derivative, Serialize, Deserialize, Encode,Decode)]
    // #[derivative(
    //     Clone(bound = "CallData: Clone"),
    //     Debug(bound = "CallData: std::fmt::Debug"),
    //     Eq(bound = "CallData: std::cmp::Eq"),
    //     Ord(bound = "CallData: std::cmp::Ord"),
    //     PartialEq(bound = "CallData: std::cmp::PartialEq"),
    //     PartialOrd(bound = "CallData: std::cmp::PartialOrd")
    // )]
    // pub struct VanePayload<CallData> {
    //     pallet_name: Cow<'static, str>,
    //     call_name: Cow<'static, str>,
    //     call_data: CallData,
    //     validation_hash: Option<[u8; 32]>,
    // }

    // impl<CallData> From<Payload<CallData>> for VanePayload<CallData> {
    //     fn from(value: Payload<CallData>) -> Self {
    //         Self { pallet_name: value.pallet_name().into(), call_name: value.call_name().into(), call_data: value.call_data(), validation_hash: None }
    //     }
    // }

    #[derive(Debug, Encode, Decode, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum ConfirmationStatus {
        WaitingForReceiver,
        WaitingForSender,
        Ready,
        Accepted,
        RejectedMismatchAddress,
        RejectedSenderRevert
    }

    impl From<TxObject> for TxConfirmationObject {
        fn from(value: TxObject) -> Self {
            Self {
                tx_id: value.tx_id,
                call: value.call,
                confirmation_status: ConfirmationStatus::WaitingForReceiver,
                network: value.network,
                confirmed_sender_address: None,
                confirmed_receiver_address: None,
                multi_id: value.multi_id,
            }
        }
    }

    /// Object to be propagated to network simulator and router layer
    #[derive(Debug, Encode, Decode, Clone, Serialize, Deserialize)]
    pub struct TxSimulationObject {
        sender_address: VaneMultiAddress<AccountId32, ()>,
        receiver_address: VaneMultiAddress<AccountId32, ()>,
        // Tx hash representation
        tx_id: String,
        // Tx function encoded
        call: VaneCallData,
        // State of the Tx to be confirmed
        confirmation_status: ConfirmationStatus,
        // blockchain network to submit the Tx to
        network: BlockchainNetwork,
    }

    impl TxSimulationObject {
        pub fn get_tx_id(self) -> String {
            self.tx_id
        }
    }

    /// Struct to be sent in the network for confirmation from sender and receiver
    #[derive(Debug, Encode, Decode, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct TxConfirmationObject {
        confirmed_sender_address: Option<VaneMultiAddress<AccountId32, ()>>,
        confirmed_receiver_address: Option<VaneMultiAddress<AccountId32, ()>>,
        // Tx hash representation and acting as a link among 3 objects (TxObject, TxSimulation, TxConfirmation)
        tx_id: String,
        pub call: VaneCallData,
        // State of the Tx to be confirmed
        confirmation_status: ConfirmationStatus,
        // multi_id
        multi_id: VaneMultiAddress<AccountId32, ()>,
        // Blockchain network to submit the Tx to
        network: BlockchainNetwork,
    }

    impl From<TxConfirmationObject> for TxSimulationObject {
        fn from(value: TxConfirmationObject) -> Self {
            Self {
                tx_id: value.tx_id,
                call: value.call,
                confirmation_status: value.confirmation_status,
                network: value.network,
                sender_address: value
                    .confirmed_sender_address
                    .expect("Failed to unwrap confirmed sender"),
                receiver_address: value
                    .confirmed_receiver_address
                    .expect("Failed to unwrap confirmed receiver"),
            }
        }
    }

    impl TxConfirmationObject {
        pub fn update_confirmation_status(&mut self, status: ConfirmationStatus) {
            self.confirmation_status = status
        }

        pub fn set_confirmed_receiver(
            &mut self,
            confirmed_receiver: VaneMultiAddress<AccountId32, ()>,
        ) {
            self.confirmed_receiver_address = Some(confirmed_receiver)
        }

        pub fn set_confirmed_sender(
            &mut self,
            confirmed_sender: VaneMultiAddress<AccountId32, ()>,
        ) {
            self.confirmed_sender_address = Some(confirmed_sender)
        }

        pub fn get_confirmation_status(&self) -> ConfirmationStatus {
            self.confirmation_status.clone()
        }

        pub fn calculate_confirmed_multi_id(
            &self,
            sender: VaneMultiAddress<AccountId32, ()>,
        ) -> MultiId {
            let receiver = self
                .confirmed_receiver_address
                .clone()
                .expect("Failed to unwrap confirmed receiver");

            let multi_id = (sender, receiver, b"VANE").using_encoded(blake2_256);
            let multi_id: VaneMultiAddress<AccountId32, ()> = VaneMultiAddress::Address32(multi_id);
            multi_id
        }

        pub fn get_multi_id(&self) -> MultiId {
            self.multi_id.clone()
        }

        pub fn get_tx_id(self) -> String {
            self.tx_id
        }
    }

    #[derive(Debug, Encode, Decode, Clone, Serialize, Deserialize)]
    pub enum LifetimeStatus {
        Valid,
        Invalid,
    }

    /// Supported networks
    #[derive(Debug, Encode, Decode, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum BlockchainNetwork {
        Polkadot,
        Kusama,
        Astar,
        Moonbeam,
        Ethereum,
        Optimism,
        Arbitrum,
        Solana,
    }

    /// Account data types
    /// This is just similar to what `MultiAddress` is but with `serde:Serialize` implemented
    #[derive(
        Encode,
        Decode,
        PartialEq,
        Eq,
        Clone,
        scale_info::TypeInfo,
        serde::Serialize,
        serde::Deserialize,
        Debug,
        Hash,
        PartialOrd,
        Ord,
    )]
    #[cfg_attr(feature = "std", derive(Hash))]
    pub enum VaneMultiAddress<AccountId, AccountIndex> {
        /// It's an account ID (pubkey).
        Id(AccountId),
        /// It's an account index.
        Index(#[codec(compact)] AccountIndex),
        /// It's some arbitrary raw bytes.
        Raw(Vec<u8>),
        /// It's a 32 byte representation.
        Address32([u8; 32]),
        /// Its a 20 byte representation.
        Address20([u8; 20]),
    }

    impl From<VaneMultiAddress<AccountId32, ()>> for MultiAddress<AccountId32, ()> {
        fn from(value: VaneMultiAddress<AccountId32, ()>) -> Self {
            match value {
                VaneMultiAddress::Address20(addr) => MultiAddress::Address20(addr),
                VaneMultiAddress::Address32(addr) => MultiAddress::Address32(addr),
                VaneMultiAddress::Id(id) => MultiAddress::Id(id),
                VaneMultiAddress::Raw(raw) => MultiAddress::Raw(raw),
                VaneMultiAddress::Index(index) => MultiAddress::Index(index),
            }
        }
    }

    impl From<MultiAddress<AccountId32, ()>> for VaneMultiAddress<AccountId32, ()> {
        fn from(value: MultiAddress<AccountId32, ()>) -> Self {
            match value {
                MultiAddress::Address20(addr) => VaneMultiAddress::Address20(addr),
                MultiAddress::Address32(addr) => VaneMultiAddress::Address32(addr),
                MultiAddress::Id(id) => VaneMultiAddress::Id(id),
                MultiAddress::Raw(raw) => VaneMultiAddress::Raw(raw),
                MultiAddress::Index(index) => VaneMultiAddress::Index(index),
            }
        }
    }
}
