use crate::money::Money;

use crate::ids::TransactionId;

pub struct Snapshot {
    pub from: TransactionId,
    pub available: Money,
    pub help: Money,
}

