use super::{Transaction, TransactionData};

use crate::ids::TransactionId;
use crate::Result;

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
        let mut transactions = vec![];

        for tx_id in self.chron.iter() {
            let (_, transaction) = self.map.get(tx_id).expect(&format!(
                "Invalid program state: self.chron contains [{tx_id}], but self.map does not."
            ));

            transactions.push(transaction);
        }

        return transactions;
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
                let (_, transaction) = self.map.get(tx_id).expect(&format!(
                    "Invalid program state: self.chron contains [{tx_id}], but self.map does not."
                ));

                transactions.push(transaction);
            }
        }

        if !found {
            return None;
        }

        return Some(transactions);
    }

    pub fn push(&mut self, transaction: Transaction) {
        let data: &TransactionData = &transaction;
        let id = data.id;

        self.map.insert(id, (self.chron.len(), transaction));
        self.chron.push(id);
    }

    pub fn replace(
        &mut self,
        id: TransactionId,
        map_fn: impl FnOnce(Transaction) -> Result<Transaction>,
    ) -> Result<Option<&Transaction>> {
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

#[cfg(test)]
mod tests {
    use crate::{
        models::{new_transaction, TransactionType},
        Money,
    };

    use super::*;

    const SOME_TRANSACTION_ID: TransactionId = TransactionId(123);
    const OTHER_TRANSACTION_ID: TransactionId = TransactionId(456);

    const SOME_TRANSACTION_TYPE: TransactionType = TransactionType::Deposit;
    const OTHER_TRANSACTION_TYPE: TransactionType = TransactionType::Withdrawal;

    const SOME_AMOUNT: Money = Money(1234);
    const OTHER_AMOUNT: Money = Money(5678);

    #[test]
    fn new_is_empty() {
        let transactions = Transactions::new();
        assert_eq!(transactions.map.is_empty(), true);
        assert_eq!(transactions.chron.is_empty(), true);
    }

    #[test]
    fn push() {
        let mut transactions = Transactions::new();

        let transaction1 = build_some_transaction();
        let data1: &TransactionData = &transaction1;
        let data1 = data1.clone();

        let transaction2 = build_other_transaction();
        let data2: &TransactionData = &transaction2;
        let data2 = data2.clone();

        transactions.push(transaction1.clone());

        assert_eq!(transactions.chron.len(), 1);
        assert_eq!(transactions.chron[0], data1.id);
        assert_eq!(transactions.map.len(), 1);

        {
            let values = transactions.map.get(&data1.id);
            assert_eq!(values.is_some(), true);

            let values = values.unwrap();
            assert_eq!(values.0, 0);
            assert_eq!(values.1, transaction1);
        }

        transactions.push(transaction2.clone());

        assert_eq!(transactions.chron.len(), 2);
        assert_eq!(transactions.chron[0], data1.id);
        assert_eq!(transactions.chron[1], data2.id);
        assert_eq!(transactions.map.len(), 2);

        {
            let values = transactions.map.get(&data1.id);
            assert_eq!(values.is_some(), true);

            let values = values.unwrap();
            assert_eq!(values.0, 0);
            assert_eq!(values.1, transaction1);
        }

        {
            let values = transactions.map.get(&data2.id);
            assert_eq!(values.is_some(), true);

            let values = values.unwrap();
            assert_eq!(values.0, 1);
            assert_eq!(values.1, transaction2);
        }
    }

    #[test]
    fn get_all() {
        let mut transactions = Transactions::new();

        let transaction1 = build_some_transaction();
        let transaction2 = build_other_transaction();

        transactions.push(transaction1.clone());

        let all = transactions.get_all();
        assert_eq!(all, vec![&transaction1]);

        transactions.push(transaction2.clone());

        let all = transactions.get_all();
        assert_eq!(all, vec![&transaction1, &transaction2]);
    }

    #[test]
    fn get_since() {
        let mut transactions = Transactions::new();

        let transaction1 = build_some_transaction();
        let transaction2 = build_other_transaction();

        transactions.push(transaction1.clone());

        {
            let since = transactions.get_since(&SOME_TRANSACTION_ID);
            assert_eq!(since.is_some(), true);
            assert_eq!(since.unwrap().is_empty(), true);
        }

        {
            let since = transactions.get_since(&OTHER_TRANSACTION_ID);
            assert_eq!(since.is_some(), false);
        }

        transactions.push(transaction2.clone());

        {
            let since = transactions.get_since(&SOME_TRANSACTION_ID);
            assert_eq!(since.is_some(), true);
            assert_eq!(since.unwrap(), vec![&transaction2]);
        }

        {
            let since = transactions.get_since(&OTHER_TRANSACTION_ID);
            assert_eq!(since.is_some(), true);
            assert_eq!(since.unwrap().is_empty(), true);
        }
    }

    #[test]
    fn replace() {
        let mut transactions = Transactions::new();

        let transaction1 = build_some_transaction();
        let transaction2 = build_other_transaction();

        let res = transactions.replace(SOME_TRANSACTION_ID, |x| Ok(x));

        assert_eq!(res.is_ok(), true);
        assert_eq!(res.unwrap().is_none(), true);

        transactions.push(transaction1.clone());
        transactions.push(transaction2.clone());

        let res = transactions.replace(SOME_TRANSACTION_ID, |x| match x {
            Transaction::Valid(x) => Ok(Transaction::Disputed(x.dispute())),
            _ => panic!("Invalid: {x:?}"),
        });

        assert_eq!(res.is_ok(), true);

        let expected = match transaction1 {
            Transaction::Valid(x) => Transaction::Disputed(x.dispute()),
            _ => panic!("Invalid: {transaction1:?}"),
        };

        let res = res.unwrap();
        assert_eq!(res.is_some(), true);
        assert_eq!(res.unwrap(), &expected);
    }

    fn build_some_transaction() -> Transaction {
        return Transaction::Valid(new_transaction(
            SOME_TRANSACTION_ID,
            SOME_TRANSACTION_TYPE,
            SOME_AMOUNT,
        ));
    }

    fn build_other_transaction() -> Transaction {
        return Transaction::Valid(new_transaction(
            OTHER_TRANSACTION_ID,
            OTHER_TRANSACTION_TYPE,
            OTHER_AMOUNT,
        ));
    }
}
