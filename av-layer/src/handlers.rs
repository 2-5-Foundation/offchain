use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use btree_slab::BTreeMap;

use crate::traits::*;

/// A mock database storing each address to the transactions each having a key
/// `address` ===> `tx_id`=====> `Vec<u8>`
pub struct MockDB(pub HashMap<String,BTreeMap<String,Vec<u8>>>);

pub struct TxSubmissionHandler {
    db: Rc<RefCell<MockDB>>
}

pub struct TxConfirmationHandler {}

pub struct ToNetworkRouterHandler {}


// ======================================================================

