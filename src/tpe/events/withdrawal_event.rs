use crate::ids::{ClientId, TransactionId};
use crate::Money;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct WithdrawalEvent {
    pub client_id: ClientId,
    pub transaction_id: TransactionId,
    pub amount: Money,
}

