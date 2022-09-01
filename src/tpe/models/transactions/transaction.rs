use super::{
    TransactionType,
    TransactionState,
};

use crate::ids::TransactionId;
use crate::money::Money;

use std::ops::Deref;

pub trait Transaction: Deref<Target = TransactionData> {
    fn get_state(&self) -> TransactionState;
}

pub struct TransactionData {
    pub id: TransactionId,
    pub transaction_type: TransactionType,
    pub amount: Money,
}

