use crate::ids::{ClientId, TransactionId};
use crate::Transaction;

use std::collections::{HashMap, HashSet};

/// Represents a WORM (Write Once, Read Many) data structure for keeping track of transactions
#[derive(Debug, Default)]
pub struct Ledger {
    history: Vec<Transaction>,
    lookup_map: HashMap<TransactionId, Vec<usize>>,
}

impl Ledger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, tx: Transaction) -> usize {
        let id = tx.id;
        let index = self.history.len();

        self.history.push(tx);

        if let Some(indicies) = self.lookup_map.get_mut(&id) {
            indicies.push(index);
        } else {
            self.lookup_map.insert(id, vec![index]);
        }

        index
    }

    pub fn invalidate(&mut self, index: &usize) -> bool {
        let index = *index;

        if index >= self.history.len() {
            return false;
        }

        self.history[index].invalid = true;

        true
    }

    pub fn get_by_index(&self, index: &usize) -> Option<&Transaction> {
        let index = *index;

        if index >= self.history.len() {
            return None;
        }

        Some(&self.history[index])
    }

    /// Returns vector of valid transactions for a transaction ID, until the given index
    pub fn get_valid_transactions_until(
        &self,
        ledger_idx: &usize,
        id: &TransactionId,
    ) -> Option<Vec<&Transaction>> {
        if let Some(indicies) = self.lookup_map.get(id) {
            let indicies = indicies.iter().copied().collect::<HashSet<usize>>();

            let transactions = self
                .history
                .iter()
                .enumerate()
                .filter_map(|(idx, tx)| {
                    if !tx.invalid && idx <= *ledger_idx && indicies.contains(&idx) {
                        return Some(tx);
                    }

                    None
                })
                .collect();

            return Some(transactions);
        }

        None
    }

    /// Finds indicies for a client ID between the given inclusive bounds
    pub fn find_indicies_for_client_id(&self, client_id: ClientId, from_idx: usize) -> Vec<usize> {
        let mut indicies = vec![];

        for (idx, tx) in self.history[from_idx..].iter().enumerate() {
            if tx.client_id == client_id {
                indicies.push(from_idx + idx);
            }
        }

        indicies
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use crate::{Money, TransactionType};

    use super::*;

    const SOME_TRANSACTION_ID: TransactionId = TransactionId(123);
    const OTHER_TRANSACTION_ID: TransactionId = TransactionId(321);

    const SOME_CLIENT_ID: ClientId = ClientId(40);
    const OTHER_CLIENT_ID: ClientId = ClientId(41);

    const SOME_AMOUNT: Money = Money(555444);

    fn build_transaction(
        id: TransactionId,
        client_id: ClientId,
        tx_type: TransactionType,
    ) -> Transaction {
        Transaction {
            id,
            client_id,
            tx_type,
            invalid: false,
        }
    }

    #[test]
    fn append() {
        let mut ledger = Ledger::new();

        let transaction1 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Deposit {
                amount: SOME_AMOUNT,
            },
        );

        ledger.append(transaction1.clone());

        assert_eq!(ledger.history, vec![transaction1.clone()]);
        assert_eq!(
            ledger.lookup_map,
            vec![(SOME_TRANSACTION_ID, vec![0])].into_iter().collect()
        );

        let transaction2 = build_transaction(
            OTHER_TRANSACTION_ID,
            OTHER_CLIENT_ID,
            TransactionType::Deposit {
                amount: SOME_AMOUNT,
            },
        );
        ledger.append(transaction2.clone());

        assert_eq!(
            ledger.history,
            vec![transaction1.clone(), transaction2.clone()]
        );
        assert_eq!(
            ledger.lookup_map,
            vec![
                (SOME_TRANSACTION_ID, vec![0]),
                (OTHER_TRANSACTION_ID, vec![1])
            ]
            .into_iter()
            .collect()
        );

        let transaction3 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Dispute,
        );
        ledger.append(transaction3.clone());

        assert_eq!(
            ledger.history,
            vec![
                transaction1,
                transaction2,
                transaction3
            ]
        );
        assert_eq!(
            ledger.lookup_map,
            vec![
                (SOME_TRANSACTION_ID, vec![0, 2]),
                (OTHER_TRANSACTION_ID, vec![1])
            ]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn invalidate() {
        let mut ledger = Ledger::new();

        let transaction1 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Deposit {
                amount: SOME_AMOUNT,
            },
        );
        ledger.append(transaction1);

        assert!(ledger.invalidate(&0));
        assert!(ledger.history[0].invalid);

        assert!(!ledger.invalidate(&1));
    }

    #[test]
    fn get_by_index() {
        let mut ledger = Ledger::new();

        let transaction1 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Deposit {
                amount: SOME_AMOUNT,
            },
        );
        ledger.append(transaction1.clone());

        assert_eq!(ledger.get_by_index(&0), Some(&transaction1));

        assert!(ledger.get_by_index(&1).is_none());
    }

    #[test]
    fn get_valid_transactions_until() {
        let mut ledger = Ledger::new();

        assert!(ledger
            .get_valid_transactions_until(&100, &SOME_TRANSACTION_ID)
            .is_none());

        let transaction1 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Deposit {
                amount: SOME_AMOUNT,
            },
        );
        ledger.append(transaction1.clone());

        let transaction2 = build_transaction(
            OTHER_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Deposit {
                amount: SOME_AMOUNT,
            },
        );
        ledger.append(transaction2);

        let transaction3 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Dispute,
        );
        ledger.append(transaction3.clone());

        assert_eq!(
            ledger.get_valid_transactions_until(&0, &SOME_TRANSACTION_ID),
            Some(vec![&transaction1])
        );
        assert_eq!(
            ledger.get_valid_transactions_until(&1, &SOME_TRANSACTION_ID),
            Some(vec![&transaction1])
        );
        assert_eq!(
            ledger.get_valid_transactions_until(&2, &SOME_TRANSACTION_ID),
            Some(vec![&transaction1, &transaction3])
        );
        assert_eq!(
            ledger.get_valid_transactions_until(&3, &SOME_TRANSACTION_ID),
            Some(vec![&transaction1, &transaction3])
        );
    }

    #[test]
    fn find_indicies_for_client_id() {
        let mut ledger = Ledger::new();

        assert!(ledger
            .find_indicies_for_client_id(SOME_CLIENT_ID, 0)
            .is_empty());

        let transaction1 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Deposit {
                amount: SOME_AMOUNT,
            },
        );
        ledger.append(transaction1);

        assert_eq!(
            ledger.find_indicies_for_client_id(SOME_CLIENT_ID, 0),
            vec![0]
        );

        let transaction2 = build_transaction(
            OTHER_TRANSACTION_ID,
            OTHER_CLIENT_ID,
            TransactionType::Deposit {
                amount: SOME_AMOUNT,
            },
        );
        ledger.append(transaction2);

        assert_eq!(
            ledger.find_indicies_for_client_id(SOME_CLIENT_ID, 0),
            vec![0]
        );

        let transaction3 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Dispute,
        );
        ledger.append(transaction3);

        assert_eq!(
            ledger.find_indicies_for_client_id(SOME_CLIENT_ID, 0),
            vec![0, 2]
        );
    }
}
