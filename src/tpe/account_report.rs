use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct AccountReport {
    pub client: String,
    pub available: String,
    pub held: String,
    pub total: String,
    pub locked: bool,
}

