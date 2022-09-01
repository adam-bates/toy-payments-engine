use super::Transaction;

use crate::ids::TransactionId;

use std::collections::{HashMap, hash_map::Values};

type TxValue = Box<dyn Transaction>;

pub struct Transactions {
    map: HashMap<TransactionId, TxValue>,
    chron: Vec<TransactionId>,
}

impl Transactions {
    pub fn new() -> Self {
        return Self {
            map: HashMap::new(),
            chron: Vec::new(),
        };
    }

    pub fn get_by_id(&self, id: &TransactionId) -> Option<&TxValue> {
        return self.map.get(id);
    }

    pub fn get_all(&self) -> Vec<&TxValue> {
        return self.map.values().collect();
    }

    pub fn get_since(&self, id: &TransactionId) -> Option<Vec<&TxValue>> {
        let mut transactions = vec![];

        let mut found = false;

        for tx_id in self.chron.iter() {
            if tx_id == id {
                found = true;
                continue;
            }

            if found {
                let transaction = self.map.get(tx_id)
                    .expect(&format!("Invalid program state: self.chron contains [{tx_id}], but self.map does not."));

                transactions.push(transaction);
            }
        }

        if !found {
            return None;
        }

        return Some(transactions);
    }

    pub fn get_since_or_all(&self, id: &TransactionId) -> Vec<&TxValue> {
        return self.get_since(id).unwrap_or_else(|| self.get_all());
    }
}

