use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccountReport {
    pub client: String,
    pub available: String,
    pub held: String,
    pub total: String,
    pub locked: bool,
}


