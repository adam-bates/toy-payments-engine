use crate::{Money, ids::TransactionId};

use thiserror::Error;

pub struct Snapshot {
    pub from: Option<TransactionId>,
    pub available: Money,
    pub held: Money,
}

#[derive(Debug, Error)]
pub enum SnapshotError {
    #[error("Invalid from: {0}")]
    InvalidFrom(TransactionId),
}

impl Snapshot {
    pub fn new() -> Self {
        return Self {
            from: None,
            available: Money(0),
            held: Money(0),
        };
    }
}

