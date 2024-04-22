pub use common::*;
use frame_support::StorageHasher;
use frame_support::Twox64Concat;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use sp_runtime::MultiAddress;
pub mod common {

    use super::*;
    /// The transaction object which all operations will be applied upon
    /// `call`: encoded transaction function call
    /// `network`: network to which the transaction will be submitted to
    /// `lifetime`: maximum period of time in minutes should this transaction be valid on confirmation phase
    /// `multi_id`: The computed address from receiver and sender, this should be kept hidden as it will be used for confirmation
    #[derive(Debug, Encode, Serialize, Deserialize, Decode, Clone)]
    pub struct TxObject {
        tx_id: String,
        pub call: Vec<u8>,
        pub sender_address: VaneMultiAddress<u128, ()>,
        pub receiver_address: VaneMultiAddress<u128, ()>,
        multi_id: VaneMultiAddress<u128, ()>,
        pub network: BlockchainNetwork,
        pub lifetime: Option<u8>,
        //submitted_time:
        pub lifetime_status: LifetimeStatus,
    }

    impl TxObject {
        pub fn new(
            call: Vec<u8>,
            sender_address: MultiAddress<u128, ()>,
            receiver_address: MultiAddress<u128, ()>,
            network: BlockchainNetwork,
        ) -> Self {
            let tx_id = Twox64Concat::hash(&call);
            let tx_id = String::from_utf8(tx_id).expect("Failed to convert tx id from bytes");

            let multi_id = Twox64Concat::hash(
                format! {"{}{}VANEMULTIID",sender_address,receiver_address}.as_bytes(),
            );
            let multi_id: MultiAddress<u128, ()> = MultiAddress::Raw(multi_id);
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

        pub fn get_multi_id(&self) -> MultiAddress<u128, ()> {
            self.multi_id.clone().into()
        }
    }

    #[derive(Debug, Encode, Decode, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum ConfirmationStatus {
        WaitingForReceiver,
        WaitingForSender,
        Ready,
        Accepted,
        Rejected,
    }

    impl From<TxObject> for TxConfirmationObject {
        fn from(value: TxObject) -> Self {
            Self {
                tx_id: value.tx_id,
                call: value.call,
                receiver_sig: None,
                sender_sig: None,
                confirmation_status: ConfirmationStatus::WaitingForReceiver,
                network: value.network,
                sender_address: value.sender_address.into(),
                receiver_address: value.receiver_address.into(),
            }
        }
    }

    /// Object to be propagated to network simulator and router layer
    #[derive(Debug, Encode, Decode, Clone, Serialize, Deserialize)]
    pub struct TxSimulationObject {
        sender_address: VaneMultiAddress<u128, ()>,
        receiver_address: VaneMultiAddress<u128, ()>,
        // Tx hash representation
        tx_id: String,
        // Tx function encoded
        call: Vec<u8>,
        // State of the Tx to be confirmed
        confirmation_status: ConfirmationStatus,
        // blockchain network to submit the Tx to
        network: BlockchainNetwork,
    }

    /// Struct to be sent in the network for confirmation from sender and receiver
    #[derive(Debug, Encode, Decode, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct TxConfirmationObject {
        sender_address: VaneMultiAddress<u128, ()>,
        receiver_address: VaneMultiAddress<u128, ()>,
        // Tx hash representation and acting as a link among 3 objects (TxObject, TxSimulation, TxConfirmation)
        tx_id: String,
        pub call: Vec<u8>,
        // State of the Tx to be confirmed
        confirmation_status: ConfirmationStatus,
        // Receiver confirmation signature
        receiver_sig: Option<Vec<u8>>,
        // Sender confirmation signature
        sender_sig: Option<Vec<u8>>,
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
                sender_address: value.sender_address,
                receiver_address: value.receiver_address,
            }
        }
    }

    impl TxConfirmationObject {
        pub fn update_confirmation_status(&mut self, status: ConfirmationStatus) {
            self.confirmation_status = status
        }

        pub fn set_receiver_sig(&mut self, receiver_sig: Vec<u8>) {
            self.receiver_sig = Some(receiver_sig)
        }

        pub fn set_sender_sig(&mut self, sender_sig: Vec<u8>) {
            self.sender_sig = Some(sender_sig)
        }

        pub fn get_confirmation_status(&self) -> ConfirmationStatus {
            self.confirmation_status.clone()
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

    impl From<VaneMultiAddress<u128, ()>> for MultiAddress<u128, ()> {
        fn from(value: VaneMultiAddress<u128, ()>) -> Self {
            match value {
                VaneMultiAddress::Address20(addr) => MultiAddress::Address20(addr),
                VaneMultiAddress::Address32(addr) => MultiAddress::Address32(addr),
                VaneMultiAddress::Id(id) => MultiAddress::Id(id),
                VaneMultiAddress::Raw(raw) => MultiAddress::Raw(raw),
                VaneMultiAddress::Index(index) => MultiAddress::Index(index),
            }
        }
    }

    impl From<MultiAddress<u128, ()>> for VaneMultiAddress<u128, ()> {
        fn from(value: MultiAddress<u128, ()>) -> Self {
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
