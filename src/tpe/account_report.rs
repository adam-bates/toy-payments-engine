use serde::{Serialize, Deserialize};

/// Report representing overview of a single account
/// Meant for easy serialization
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccountReport {
    pub client: String,
    pub available: String,
    pub held: String,
    pub total: String,
    pub locked: bool,
}


