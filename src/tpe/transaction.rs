use crate::ids::{ClientId, TransactionId};
use crate::Money;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub id: TransactionId,
    pub client_id: ClientId,
    pub tx_type: TransactionType,
    pub invalid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionType {
    Deposit { amount: Money },
    Withdrawal { amount: Money },
    Dispute,
    Resolve,
    ChargeBack,
}
