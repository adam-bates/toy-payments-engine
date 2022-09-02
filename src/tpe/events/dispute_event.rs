use crate::ids::{ClientId, TransactionId};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct DisputeEvent {
    pub client_id: ClientId,
    pub transaction_id: TransactionId,
}

