use crate::ids::{ClientId, TransactionId};

pub struct ChargeBackEvent {
    pub client_id: ClientId,
    pub transaction_id: TransactionId,
}
