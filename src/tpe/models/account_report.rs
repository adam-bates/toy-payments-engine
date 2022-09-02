use crate::ids::ClientId;
use crate::Money;

#[derive(Debug)]
pub struct AccountReport {
    pub client: ClientId,
    pub available: Money,
    pub held: Money,
    pub total: Money,
    pub locked: bool,
}

