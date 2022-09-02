use crate::ids::{ClientId, TransactionId};

#[derive(Debug)]
pub struct DisputeEvent {
    pub client_id: ClientId,
    pub transaction_id: TransactionId,
}

