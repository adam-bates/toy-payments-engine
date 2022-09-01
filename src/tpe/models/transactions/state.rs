use super::{
    Transaction,
    TransactionData,
    TransactionType,
};

use crate::ids::TransactionId;
use crate::money::Money;

use std::ops::Deref;

pub fn new_transaction(
    id: TransactionId,
    transaction_type: TransactionType,
    amount: Money
    ) -> ValidTransaction {
    return ValidTransaction(TransactionData {
        id,
        transaction_type,
        amount,
    });
}

pub enum TransactionState {
    Valid,
    Disputed,
    ChargedBack,
}

pub struct ValidTransaction(TransactionData);

impl ValidTransaction {
    pub fn dispute(self) -> DisputedTransaction {
        return DisputedTransaction(self.0);
    }
}

impl Transaction for ValidTransaction {
    fn get_state(&self) -> TransactionState {
        return TransactionState::Valid;
    }
}

impl Deref for ValidTransaction {
    type Target = TransactionData;

    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

pub struct DisputedTransaction(TransactionData);

impl DisputedTransaction {
    pub fn resolve(self) -> ValidTransaction {
        return ValidTransaction(self.0);
    }

    pub fn charge_back(self) -> ChargedBackTransaction {
        return ChargedBackTransaction(self.0);
    }
}

impl Transaction for DisputedTransaction {
    fn get_state(&self) -> TransactionState {
        return TransactionState::Disputed;
    }
}

impl Deref for DisputedTransaction {
    type Target = TransactionData;

    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

pub struct ChargedBackTransaction(TransactionData);

impl Transaction for ChargedBackTransaction {
    fn get_state(&self) -> TransactionState {
        return TransactionState::ChargedBack;
    }
}

impl Deref for ChargedBackTransaction {
    type Target = TransactionData;

    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

