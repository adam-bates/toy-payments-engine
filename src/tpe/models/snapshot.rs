use crate::{Money, ids::TransactionId};

#[derive(Debug, Clone)]
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

