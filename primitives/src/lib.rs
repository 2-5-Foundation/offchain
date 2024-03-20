
pub use common::*;

pub mod common {

    pub struct TxObject {
        sender_address: String,
        receiver_address: String,
        network: BlockchainNetwork,

    }


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