use crate::ids::{ClientId, TransactionId};

#[derive(Debug)]
pub struct ChargeBackEvent {
    pub client_id: ClientId,
    pub transaction_id: TransactionId,
}
