use super::{Snapshot, Transactions};

use crate::ids::ClientId;

#[derive(Debug)]
pub struct Account {
    pub client_id: ClientId,
    pub transactions: Transactions,
    pub snapshot: Snapshot,
}

impl Account {
    pub fn new(client_id: ClientId) -> Self {
        Self {
            client_id,
            transactions: Transactions::new(),
            snapshot: Snapshot::new(),
        }
    }
}
