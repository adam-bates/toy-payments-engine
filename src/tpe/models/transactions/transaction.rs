use super::TransactionType;

use crate::ids::TransactionId;
use crate::money::Money;

use std::ops::Deref;

/// Creates a new ValidTransaction.
pub fn new_transaction(
    id: TransactionId,
    transaction_type: TransactionType,
    amount: Money,
) -> ValidTransaction {
    return ValidTransaction(TransactionData {
        id,
        transaction_type,
        amount,
    });
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransactionData {
    pub id: TransactionId,
    pub transaction_type: TransactionType,
    pub amount: Money,
}

/// Transaction is a finite state-machine with the following structure:
///
/// ValidTransaction
/// -> dispute: DisputedTransaction
///
/// DisputedTransaction
/// -> resolve: ValidTransaction
/// -> charge_back: ChargedBackTransaction
///
/// ChargedBackTransaction
/// -> _
#[derive(Debug, Clone, PartialEq)]
pub enum Transaction {
    Valid(ValidTransaction),
    Disputed(DisputedTransaction),
    ChargedBack(ChargedBackTransaction),
}
impl Deref for Transaction {
    type Target = TransactionData;

    fn deref(&self) -> &Self::Target {
        return match self {
            Self::Valid(transaction) => &transaction,
            Self::Disputed(transaction) => &transaction,
            Self::ChargedBack(transaction) => &transaction,
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidTransaction(TransactionData);
impl ValidTransaction {
    pub fn dispute(self) -> DisputedTransaction {
        return DisputedTransaction(self.0);
    }
}
impl Deref for ValidTransaction {
    type Target = TransactionData;

    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisputedTransaction(TransactionData);
impl DisputedTransaction {
    pub fn resolve(self) -> ValidTransaction {
        return ValidTransaction(self.0);
    }

    pub fn charge_back(self) -> ChargedBackTransaction {
        return ChargedBackTransaction(self.0);
    }
}
impl Deref for DisputedTransaction {
    type Target = TransactionData;

    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChargedBackTransaction(TransactionData);
impl Deref for ChargedBackTransaction {
    type Target = TransactionData;

    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

