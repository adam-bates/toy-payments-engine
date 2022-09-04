use crate::ids::{ClientId, TransactionId};
use crate::Result;
use crate::{AccountReport, Ledger, TransactionType};
use crate::{Money, Transaction};

use thiserror::Error;

/// Represents a snapshot in time for a given account
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountSnapshot {
    from_ledger_idx: Option<usize>,
    client_id: ClientId,
    available: Money,
    held: Money,
    locked: bool,
}

#[derive(Error, Debug)]
pub enum AccountTransactionError {
    #[error("Invalid internal ledger state: {0}")]
    InvalidLedgerState(String),

    #[error(
        "Invalid client ID: Transaction {0} for client {1} cannot be processed for client {2}"
    )]
    InvalidClientId(TransactionId, ClientId, ClientId),

    #[error("Account locked for client {0}, cannot process transaction {1}")]
    AccountLocked(ClientId, TransactionId),

    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),

    #[error("Invalid deposit attempt: {0}")]
    InvalidDeposit(String),

    #[error("Invalid withdrawal attempt: {0}")]
    InvalidWithdrawal(String),

    #[error("Invalid dispute attempt: {0}")]
    InvalidDispute(String),

    #[error("Invalid resolve attempt: {0}")]
    InvalidResolve(String),

    #[error("Invalid charge back attempt: {0}")]
    InvalidChargeBack(String),
}

impl AccountSnapshot {
    pub fn new(client_id: ClientId) -> Self {
        Self {
            from_ledger_idx: None,
            client_id,
            available: Money(0),
            held: Money(0),
            locked: false,
        }
    }

    pub fn parse_report(&self) -> Result<AccountReport> {
        let mut total = self.available;
        total.add(&self.held)?;

        Ok(AccountReport {
            client: self.client_id.to_string(),
            available: self.available.to_string(),
            held: self.held.to_string(),
            total: total.to_string(),
            locked: self.locked,
        })
    }

    /// Attempts to apply any new transactions that have been added to the ledger since the last
    /// time this was run.
    ///
    /// Any failed attempts will return an error, but the snapshot will remember where it failed,
    /// and move on to the next transaction if this function is run again.
    pub fn apply_transactions(&mut self, ledger: &mut Ledger) -> Result {
        let min_idx_unseen = self.from_ledger_idx.map(|idx| idx + 1).unwrap_or(0);

        let ledger_idicies = ledger.get_valid_indicies_for_client(self.client_id, min_idx_unseen);

        log::debug!(
            "get_valid_indicies_for_client({}, {min_idx_unseen}) = {ledger_idicies:?}",
            self.client_id
        );

        for ledger_idx in ledger_idicies {
            let res = self.apply_transaction(ledger, &ledger_idx);

            self.from_ledger_idx = Some(ledger_idx);

            if let Err(e) = res {
                ledger.invalidate(&ledger_idx);
                return Err(e);
            }
        }

        Ok(())
    }

    fn apply_transaction(&mut self, ledger: &Ledger, ledger_idx: &usize) -> Result {
        let tx = ledger.get_by_index(ledger_idx).ok_or_else(|| {
            AccountTransactionError::TransactionNotFound(format!(
                "No transaction found at ledger index: {ledger_idx}"
            ))
        })?;

        if self.locked {
            Err(AccountTransactionError::AccountLocked(
                self.client_id,
                tx.id,
            ))?;
        }

        let mut transactions = ledger.get_valid_transactions_until(ledger_idx, &tx.id);

        // already referencing latest transaction as `tx`
        transactions.pop();

        match tx.tx_type {
            TransactionType::Deposit { amount } => {
                self.apply_deposit(&mut transactions, tx, amount)?
            }
            TransactionType::Withdrawal { amount } => {
                self.apply_withdrawal(&mut transactions, tx, amount)?
            }

            TransactionType::Dispute => self.apply_dispute(&mut transactions, tx)?,
            TransactionType::Resolve => self.apply_resolve(&mut transactions, tx)?,
            TransactionType::ChargeBack => self.apply_charge_back(&mut transactions, tx)?,
        }

        Ok(())
    }

    fn apply_deposit(
        &mut self,
        transactions: &mut Vec<&Transaction>,
        tx: &Transaction,
        amount: Money,
    ) -> Result {
        if transactions.len() > 1 {
            Err(AccountTransactionError::InvalidDeposit(format!(
                "Duplicate transaction ID found: {}",
                tx.id
            )))?;
        }

        self.available.add(&amount)?;

        Ok(())
    }

    fn apply_withdrawal(
        &mut self,
        transactions: &mut Vec<&Transaction>,
        tx: &Transaction,
        amount: Money,
    ) -> Result {
        if transactions.len() > 1 {
            Err(AccountTransactionError::InvalidWithdrawal(format!(
                "Duplicate transaction ID found: {}",
                tx.id
            )))?;
        }

        if self.available.0 < amount.0 {
            Err(AccountTransactionError::InvalidWithdrawal(format!(
                "Cannot withdraw {} from client {} when available amount is {}",
                amount, tx.client_id, self.available
            )))?
        }

        self.available.sub(&amount)?;

        Ok(())
    }

    fn apply_dispute(&mut self, transactions: &mut Vec<&Transaction>, tx: &Transaction) -> Result {
        let og = self.get_expected_original(
            transactions,
            &tx.id,
            &AccountTransactionError::InvalidDispute,
        )?;

        match og.tx_type {
            TransactionType::Deposit { amount } => {
                let mut available = self.available.clone();
                let mut held = self.held.clone();

                available.sub(&amount)?;
                held.add(&amount)?;

                // Only apply if both operations were successful
                self.available = available;
                self.held = held;
            }

            // Not sure if possible to dispute withdrawals.
            // Assuming that you cannot, based on the term "ChargeBack"

            // TransactionType::Withdrawal { amount } => { .. },
            _ => Err(AccountTransactionError::InvalidDispute(format!(
                "Cannot dispute a transaction of type: {:?}",
                og.tx_type
            )))?,
        }

        Ok(())
    }

    fn apply_resolve(&mut self, transactions: &mut Vec<&Transaction>, tx: &Transaction) -> Result {
        let prev = self.get_expected_prev(
            transactions,
            &tx.id,
            &AccountTransactionError::InvalidResolve,
        )?;

        let og = match prev.tx_type {
            TransactionType::Dispute => self.get_expected_original(
                transactions,
                &tx.id,
                &AccountTransactionError::InvalidResolve,
            )?,

            _ => Err(AccountTransactionError::InvalidResolve(format!(
                "Cannot resolve a transaction of type: {:?}",
                prev.tx_type
            )))?,
        };

        match og.tx_type {
            TransactionType::Deposit { amount } => {
                let mut available = self.available.clone();
                let mut held = self.held.clone();

                held.sub(&amount)?;
                available.add(&amount)?;

                // Only apply if both operations were successful
                self.available = available;
                self.held = held;
            }
            _ => Err(AccountTransactionError::InvalidLedgerState(format!(
                "Cannot find deposit to resolve for transaction ID: {}",
                tx.id,
            )))?,
        }

        Ok(())
    }

    fn apply_charge_back(
        &mut self,
        transactions: &mut Vec<&Transaction>,
        tx: &Transaction,
    ) -> Result {
        let prev = self.get_expected_prev(
            transactions,
            &tx.id,
            &AccountTransactionError::InvalidChargeBack,
        )?;

        let og = match prev.tx_type {
            TransactionType::Dispute => self.get_expected_original(
                transactions,
                &tx.id,
                &AccountTransactionError::InvalidChargeBack,
            )?,

            _ => Err(AccountTransactionError::InvalidChargeBack(format!(
                "Cannot charge back a transaction of type: {:?}",
                prev.tx_type
            )))?,
        };

        match og.tx_type {
            TransactionType::Deposit { amount } => {
                self.held.sub(&amount)?;
                self.locked = true;
            }
            _ => Err(AccountTransactionError::InvalidLedgerState(format!(
                "Cannot find deposit to charge back for transaction ID: {}",
                tx.id,
            )))?,
        }

        Ok(())
    }

    /// Gets the original transaction
    /// Fails if no previous transactions, different client_ids, or invalid internal state
    fn get_expected_original<'a>(
        &self,
        transactions: &mut Vec<&'a Transaction>,
        tx_id: &TransactionId,
        err_gen: &dyn Fn(String) -> AccountTransactionError,
    ) -> Result<&'a Transaction> {
        let mut prev = self.get_expected_prev(transactions, tx_id, err_gen)?;

        while prev.tx_type == TransactionType::Resolve {
            transactions.pop();

            prev = transactions.pop().ok_or_else(|| {
                AccountTransactionError::InvalidLedgerState(format!(
                    "No previous transaction found with ID: {}",
                    tx_id
                ))
            })?;
        }

        return Ok(prev);
    }

    /// Gets the previous transaction, ensuring the same client_id
    /// Fails if no previous transaction or different client_ids
    fn get_expected_prev<'a>(
        &self,
        transactions: &mut Vec<&'a Transaction>,
        tx_id: &TransactionId,
        err_gen: &dyn Fn(String) -> AccountTransactionError,
    ) -> Result<&'a Transaction> {
        let prev = transactions
            .pop()
            .ok_or_else(|| err_gen(format!("No previous transaction found with ID: {}", tx_id)))?;

        if prev.client_id != self.client_id {
            Err(AccountTransactionError::InvalidClientId(
                prev.id,
                prev.client_id,
                self.client_id,
            ))?;
        }

        return Ok(prev);
    }
}

#[cfg(test)]
mod tests {
    use crate::{ids::TransactionId, Money, Transaction};

    use super::*;

    const SOME_TRANSACTION_ID: TransactionId = TransactionId(123);
    const OTHER_TRANSACTION_ID: TransactionId = TransactionId(321);

    const SOME_CLIENT_ID: ClientId = ClientId(40);
    const OTHER_CLIENT_ID: ClientId = ClientId(41);

    const SOME_AMOUNT: Money = Money(555444);
    const OTHER_AMOUNT: Money = Money(1000);

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

    fn build_ledger(transactions: Vec<Transaction>) -> Ledger {
        let mut ledger = Ledger::new();

        for tx in transactions.into_iter() {
            ledger.append(tx);
        }

        ledger
    }

    #[test]
    fn apply_deposit() {
        let mut snapshot1 = AccountSnapshot::new(SOME_CLIENT_ID);
        let mut snapshot2 = AccountSnapshot::new(OTHER_CLIENT_ID);

        let transaction1 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Deposit {
                amount: SOME_AMOUNT,
            },
        );

        let transaction2 = build_transaction(
            OTHER_TRANSACTION_ID,
            OTHER_CLIENT_ID,
            TransactionType::Deposit {
                amount: OTHER_AMOUNT,
            },
        );

        let mut ledger = build_ledger(vec![transaction1, transaction2]);

        let res = snapshot1.apply_transactions(&mut ledger);
        assert!(res.is_ok());

        let res = snapshot2.apply_transactions(&mut ledger);
        assert!(res.is_ok());

        assert_eq!(
            snapshot1,
            AccountSnapshot {
                client_id: SOME_CLIENT_ID,
                from_ledger_idx: Some(0),
                available: SOME_AMOUNT,
                held: Money(0),
                locked: false,
            }
        );
        assert_eq!(
            snapshot2,
            AccountSnapshot {
                client_id: OTHER_CLIENT_ID,
                from_ledger_idx: Some(1),
                available: OTHER_AMOUNT,
                held: Money(0),
                locked: false,
            }
        );
    }

    #[test]
    fn apply_withdrawal() {
        let mut snapshot = AccountSnapshot::new(SOME_CLIENT_ID);

        let transaction1 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Deposit {
                amount: SOME_AMOUNT,
            },
        );

        let transaction2 = build_transaction(
            OTHER_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Withdrawal {
                amount: SOME_AMOUNT,
            },
        );

        let mut ledger = build_ledger(vec![transaction1, transaction2]);

        let res = snapshot.apply_transactions(&mut ledger);
        assert!(res.is_ok());

        assert_eq!(
            snapshot,
            AccountSnapshot {
                client_id: SOME_CLIENT_ID,
                from_ledger_idx: Some(1),
                available: Money(0),
                held: Money(0),
                locked: false,
            }
        );
    }

    #[test]
    fn apply_dispute() {
        let mut snapshot = AccountSnapshot::new(SOME_CLIENT_ID);

        let transaction1 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Deposit {
                amount: SOME_AMOUNT,
            },
        );

        let transaction2 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Dispute,
        );

        let mut ledger = build_ledger(vec![transaction1, transaction2]);

        let res = snapshot.apply_transactions(&mut ledger);
        assert!(res.is_ok());

        assert_eq!(
            snapshot,
            AccountSnapshot {
                client_id: SOME_CLIENT_ID,
                from_ledger_idx: Some(1),
                available: Money(0),
                held: SOME_AMOUNT,
                locked: false,
            }
        );
    }

    #[test]
    fn apply_resolve() {
        let mut snapshot = AccountSnapshot::new(SOME_CLIENT_ID);

        let transaction1 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Deposit {
                amount: SOME_AMOUNT,
            },
        );

        let transaction2 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Dispute,
        );

        let transaction3 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Resolve,
        );

        let mut ledger = build_ledger(vec![transaction1, transaction2, transaction3]);

        let res = snapshot.apply_transactions(&mut ledger);
        assert!(res.is_ok());

        assert_eq!(
            snapshot,
            AccountSnapshot {
                client_id: SOME_CLIENT_ID,
                from_ledger_idx: Some(2),
                available: SOME_AMOUNT,
                held: Money(0),
                locked: false,
            }
        );
    }

    #[test]
    fn apply_charge_back() {
        let mut snapshot = AccountSnapshot::new(SOME_CLIENT_ID);

        let transaction1 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Deposit {
                amount: SOME_AMOUNT,
            },
        );

        let transaction2 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Dispute,
        );

        let transaction3 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::ChargeBack,
        );

        let mut ledger = build_ledger(vec![transaction1, transaction2, transaction3]);

        let res = snapshot.apply_transactions(&mut ledger);
        assert!(res.is_ok());

        assert_eq!(
            snapshot,
            AccountSnapshot {
                client_id: SOME_CLIENT_ID,
                from_ledger_idx: Some(2),
                available: Money(0),
                held: Money(0),
                locked: true,
            }
        );
    }

    #[test]
    fn fail_to_withdrawal_more_than_available() {
        let mut snapshot = AccountSnapshot::new(SOME_CLIENT_ID);

        let transaction1 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Deposit {
                amount: SOME_AMOUNT,
            },
        );

        let mut ledger = build_ledger(vec![transaction1]);

        let res = snapshot.apply_transactions(&mut ledger);
        assert!(res.is_ok());

        let mut invalid_amount = SOME_AMOUNT;
        invalid_amount.add(&Money(1)).unwrap();

        let transaction2 = build_transaction(
            OTHER_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Withdrawal {
                amount: invalid_amount,
            },
        );
        ledger.append(transaction2);

        let res = snapshot.apply_transactions(&mut ledger);
        assert!(res.is_err());

        assert_eq!(
            snapshot,
            AccountSnapshot {
                client_id: SOME_CLIENT_ID,
                from_ledger_idx: Some(1),
                available: SOME_AMOUNT,
                held: Money(0),
                locked: false,
            }
        );
    }

    #[test]
    fn fail_to_dispute_withdrawal() {
        let mut snapshot = AccountSnapshot::new(SOME_CLIENT_ID);

        let transaction1 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Deposit {
                amount: SOME_AMOUNT,
            },
        );
        let transaction2 = build_transaction(
            OTHER_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Withdrawal {
                amount: SOME_AMOUNT,
            },
        );

        let mut ledger = build_ledger(vec![transaction1, transaction2]);

        let res = snapshot.apply_transactions(&mut ledger);
        assert!(res.is_ok());

        let transaction3 = build_transaction(
            OTHER_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Dispute,
        );
        ledger.append(transaction3);

        let res = snapshot.apply_transactions(&mut ledger);
        assert!(res.is_err());

        assert_eq!(
            snapshot,
            AccountSnapshot {
                client_id: SOME_CLIENT_ID,
                from_ledger_idx: Some(2),
                available: Money(0),
                held: Money(0),
                locked: false,
            }
        );
    }

    #[test]
    fn fail_to_dispute_invalid_client_id() {
        let mut snapshot1 = AccountSnapshot::new(SOME_CLIENT_ID);
        let mut snapshot2 = AccountSnapshot::new(OTHER_CLIENT_ID);

        let transaction1 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Deposit {
                amount: SOME_AMOUNT,
            },
        );
        let mut ledger = build_ledger(vec![transaction1]);

        let res = snapshot1.apply_transactions(&mut ledger);
        assert!(res.is_ok());

        let res = snapshot2.apply_transactions(&mut ledger);
        assert!(res.is_ok());

        let transaction2 = build_transaction(
            SOME_TRANSACTION_ID,
            OTHER_CLIENT_ID,
            TransactionType::Dispute,
        );
        ledger.append(transaction2);

        let res = snapshot1.apply_transactions(&mut ledger);
        assert!(res.is_ok());

        let res = snapshot2.apply_transactions(&mut ledger);
        assert!(res.is_err());

        assert_eq!(
            snapshot1,
            AccountSnapshot {
                client_id: SOME_CLIENT_ID,
                from_ledger_idx: Some(0),
                available: SOME_AMOUNT,
                held: Money(0),
                locked: false,
            }
        );

        assert_eq!(
            snapshot2,
            AccountSnapshot {
                client_id: OTHER_CLIENT_ID,
                from_ledger_idx: Some(1),
                available: Money(0),
                held: Money(0),
                locked: false,
            }
        );
    }

    #[test]
    fn fail_to_deposit_on_locked_account() {
        let mut snapshot = AccountSnapshot::new(SOME_CLIENT_ID);

        let transaction1 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Deposit {
                amount: SOME_AMOUNT,
            },
        );

        let transaction2 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Dispute,
        );

        let transaction3 = build_transaction(
            SOME_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::ChargeBack,
        );

        let mut ledger = build_ledger(vec![transaction1, transaction2, transaction3]);

        let res = snapshot.apply_transactions(&mut ledger);
        assert!(res.is_ok());

        let transaction4 = build_transaction(
            OTHER_TRANSACTION_ID,
            SOME_CLIENT_ID,
            TransactionType::Deposit {
                amount: OTHER_AMOUNT,
            },
        );
        ledger.append(transaction4);

        let res = snapshot.apply_transactions(&mut ledger);
        assert!(res.is_err());

        assert_eq!(
            snapshot,
            AccountSnapshot {
                client_id: SOME_CLIENT_ID,
                from_ledger_idx: Some(3),
                available: Money(0),
                held: Money(0),
                locked: true,
            }
        );
    }
}
