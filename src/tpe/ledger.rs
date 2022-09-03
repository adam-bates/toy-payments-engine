use crate::ids::TransactionId;
use crate::Transaction;

use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct Ledger {
    history: Vec<Transaction>,
    lookup_map: HashMap<TransactionId, Vec<usize>>,
}

impl Ledger {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn append(&mut self, tx: Transaction) -> usize {
        let id = tx.id;

        if let Some(indicies) = self.lookup_map.get_mut(&id) {
            let index = self.history.len();

            self.history.push(tx);
            indicies.push(index);

            return index;
        } else {
            return self.append_new(tx);
        }
    }

    fn append_new(&mut self, tx: Transaction) -> usize {
        let id = tx.id;
        let index = self.history.len();

        self.history.push(tx);
        self.lookup_map.insert(id, vec![index]);

        return index;
    }

    pub fn invalidate(&mut self, index: &usize) -> bool {
        let index = *index;

        if index >= self.history.len() {
            return false;
        }

        self.history[index].invalid = true;

        return true;
    }

    pub fn get_by_index(&self, index: &usize) -> Option<&Transaction> {
        let index = *index;

        if index >= self.history.len() {
            return None;
        }

        return Some(&self.history[index]);
    }

    pub fn get_valid_transactions(&self, id: &TransactionId) -> Option<Vec<&Transaction>> {
        if let Some(indicies) = self.lookup_map.get(id) {
            let indicies = indicies.iter().map(|idx| *idx).collect::<HashSet<usize>>();

            let transactions = self
                .history
                .iter()
                .enumerate()
                .filter_map(|(idx, tx)| {
                    if !tx.invalid && indicies.contains(&idx) {
                        return Some(tx);
                    }

                    return None;
                })
                .collect();

            return Some(transactions);
        }

        return None;
    }

    pub fn len(&self) -> usize {
        return self.history.len();
    }
}
