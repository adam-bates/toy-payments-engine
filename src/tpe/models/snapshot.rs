use crate::{Money, ids::TransactionId};

/// Represents an snapshot in time of an account's values, with the `from` field being the most recent transactionId to
/// update the snapshot's values
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub from: Option<TransactionId>,
    pub available: Money,
    pub held: Money,
    pub locked: bool,
}

impl Snapshot {
    pub fn new() -> Self {
        return Self {
            from: None,
            available: Money(0),
            held: Money(0),
            locked: false,
        };
    }
}

