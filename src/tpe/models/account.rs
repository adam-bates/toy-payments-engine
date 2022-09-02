use super::{Snapshot, Transactions};

use crate::ids::ClientId;

pub struct Account {
    pub client_id: ClientId,
    pub transactions: Transactions,
    pub snapshot: Snapshot,
}
