use crate::ids::{ClientId, TransactionId};

#[derive(Debug)]
pub struct ResolveEvent {
    pub client_id: ClientId,
    pub transaction_id: TransactionId,
}

