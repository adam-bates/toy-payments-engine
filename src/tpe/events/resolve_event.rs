use crate::ids::{ClientId, TransactionId};

pub struct ResolveEvent {
    pub client_id: ClientId,
    pub transaction_id: TransactionId,
}

