use crate::ids::{ClientId, TransactionId};

pub struct DisputeEvent {
    pub client_id: ClientId,
    pub transaction_id: TransactionId,
}

