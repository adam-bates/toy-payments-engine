use super::{Transaction, TransactionData};

use crate::Result;
use crate::ids::TransactionId;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Transactions {
    map: HashMap<TransactionId, (usize, Transaction)>,
    chron: Vec<TransactionId>,
}

impl Transactions {
    pub fn new() -> Self {
        return Self {
            map: HashMap::new(),
            chron: Vec::new(),
        };
    }

    pub fn get_by_id(&self, id: &TransactionId) -> Option<&Transaction> {
        return self.map.get(id).map(|v| &v.1);
    }

    pub fn get_all(&self) -> Vec<&Transaction> {
        return self.map.values().map(|v| &v.1).collect();
    }

    pub fn get_since(&self, id: &TransactionId) -> Option<Vec<&Transaction>> {
        let mut transactions = vec![];

        let mut found = false;

        for tx_id in self.chron.iter() {
            if tx_id == id {
                found = true;
                continue;
            }

            if found {
                let (_, transaction) = self.map.get(tx_id)
                    .expect(&format!("Invalid program state: self.chron contains [{tx_id}], but self.map does not."));

                transactions.push(transaction);
            }
        }

        if !found {
            return None;
        }

        return Some(transactions);
    }

    pub fn get_since_or_all(&self, id: &TransactionId) -> Vec<&Transaction> {
        return self.get_since(id).unwrap_or_else(|| self.get_all());
    }

    pub fn push(&mut self, transaction: Transaction) {
        let data: &TransactionData = &transaction;
        let id = data.id;

        self.map.insert(id, (self.chron.len(), transaction));
        self.chron.push(id);
    }

    pub fn replace(&mut self, id: TransactionId, map_fn: impl FnOnce(Transaction) -> Result<Transaction>) -> Result<Option<&Transaction>> {
        let (idx, transaction) = if let Some(values) = self.map.remove(&id) {
            values
        } else {
            return Ok(None);
        };

        let transaction = map_fn(transaction)?;

        self.map.insert(id, (idx, transaction));

        return Ok(self.get_by_id(&id));
    }
}

