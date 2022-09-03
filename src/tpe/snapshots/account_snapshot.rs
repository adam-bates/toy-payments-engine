use crate::ids::{ClientId, TransactionId};
use crate::Money;
use crate::Result;
use crate::{AccountReport, Ledger, TransactionType};

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct AccountSnapshot {
    from_ledger_idx: Option<usize>,
    id: ClientId,
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
        return Self {
            from_ledger_idx: None,
            id: client_id,
            available: Money(0),
            held: Money(0),
            locked: false,
        };
    }

    pub fn apply_transaction(&mut self, ledger: &mut Ledger, ledger_idx: &usize) -> Result {
        match self.inner_apply_transaction(ledger, ledger_idx) {
            Ok(_) => return Ok(()),
            Err(e) => {
                ledger.invalidate(ledger_idx);
                return Err(e);
            }
        }
    }

    fn inner_apply_transaction(&mut self, ledger: &Ledger, ledger_idx: &usize) -> Result {
        let tx = ledger.get_by_index(ledger_idx).ok_or_else(|| {
            AccountTransactionError::TransactionNotFound(format!(
                "No transaction found at ledger index: {ledger_idx}"
            ))
        })?;

        if self.locked {
            Err(AccountTransactionError::AccountLocked(self.id, tx.id))?;
        }

        let mut transactions = ledger.get_valid_transactions(&tx.id).ok_or_else(|| {
            AccountTransactionError::TransactionNotFound(format!(
                "No transactions found in ledger for transaction ID: {}",
                tx.id
            ))
        })?;

        let tx = transactions.pop().ok_or_else(|| {
            AccountTransactionError::TransactionNotFound(format!(
                "No transactions found in ledger for transaction ID: {}",
                tx.id
            ))
        })?;

        match tx.tx_type {
            TransactionType::Deposit { amount } => {
                if transactions.len() > 1 {
                    Err(AccountTransactionError::InvalidDeposit(format!(
                        "Duplicate transaction ID found: {}",
                        tx.id
                    )))?;
                }

                self.available.add(&amount)?;
            }

            TransactionType::Withdrawal { amount } => {
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
            }

            TransactionType::Dispute => {
                let mut prev = transactions.pop().ok_or_else(|| {
                    AccountTransactionError::InvalidDispute(format!(
                        "No previous transaction found with ID: {}",
                        tx.id
                    ))
                })?;

                if prev.client_id != self.id {
                    Err(AccountTransactionError::InvalidClientId(
                        prev.id,
                        prev.client_id,
                        self.id,
                    ))?;
                }

                while prev.tx_type == TransactionType::Resolve {
                    transactions.pop(); // dispute

                    // either a resolve, or the transaction being disputed
                    prev = transactions.pop().ok_or_else(|| {
                        AccountTransactionError::InvalidLedgerState(format!(
                            "No previous transaction found with ID: {}",
                            tx.id
                        ))
                    })?;
                }

                match prev.tx_type {
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
                        prev.tx_type
                    )))?,
                }
            }

            TransactionType::Resolve => {
                let prev = transactions.pop().ok_or_else(|| {
                    AccountTransactionError::InvalidResolve(format!(
                        "No previous transaction found with ID: {}",
                        tx.id
                    ))
                })?;

                if prev.client_id != self.id {
                    Err(AccountTransactionError::InvalidClientId(
                        prev.id,
                        prev.client_id,
                        self.id,
                    ))?;
                }

                match prev.tx_type {
                    TransactionType::Dispute => {
                        let mut prev = transactions.pop().ok_or_else(|| {
                            AccountTransactionError::InvalidResolve(format!(
                                "No previous transaction found with ID: {}",
                                tx.id
                            ))
                        })?;

                        while prev.tx_type == TransactionType::Resolve {
                            transactions.pop();

                            prev = transactions.pop().ok_or_else(|| {
                                AccountTransactionError::InvalidLedgerState(format!(
                                    "No previous transaction found with ID: {}",
                                    tx.id
                                ))
                            })?;
                        }

                        match prev.tx_type {
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
                                "Cannot find deposit to resolve"
                            )))?,
                        }
                    }
                    _ => Err(AccountTransactionError::InvalidResolve(format!(
                        "Cannot resolve a transaction of type: {:?}",
                        prev.tx_type
                    )))?,
                }
            }

            TransactionType::ChargeBack => {
                let prev = transactions.pop().ok_or_else(|| {
                    AccountTransactionError::InvalidChargeBack(format!(
                        "No previous transaction found with ID: {}",
                        tx.id
                    ))
                })?;

                if prev.client_id != self.id {
                    Err(AccountTransactionError::InvalidClientId(
                        prev.id,
                        prev.client_id,
                        self.id,
                    ))?;
                }

                match prev.tx_type {
                    TransactionType::Dispute => {
                        let mut prev = transactions.pop().ok_or_else(|| {
                            AccountTransactionError::InvalidChargeBack(format!(
                                "No previous transaction found with ID: {}",
                                tx.id
                            ))
                        })?;

                        while prev.tx_type == TransactionType::Resolve {
                            transactions.pop();

                            prev = transactions.pop().ok_or_else(|| {
                                AccountTransactionError::InvalidLedgerState(format!(
                                    "No previous transaction found with ID: {}",
                                    tx.id
                                ))
                            })?;
                        }

                        match prev.tx_type {
                            TransactionType::Deposit { amount } => {
                                self.held.sub(&amount)?;
                                self.locked = true;
                            }
                            _ => Err(AccountTransactionError::InvalidLedgerState(format!(
                                "Cannot find deposit to charge back"
                            )))?,
                        }
                    }
                    _ => Err(AccountTransactionError::InvalidChargeBack(format!(
                        "Cannot resolve a transaction of type: {:?}",
                        prev.tx_type
                    )))?,
                }
            }
        }

        self.from_ledger_idx = Some(*ledger_idx);

        Ok(())
    }

    pub fn parse_report(&self) -> Result<AccountReport> {
        let mut total = self.available;
        total.add(&self.held)?;

        return Ok(AccountReport {
            client: self.id.to_string(),
            available: self.available.to_string(),
            held: self.held.to_string(),
            total: total.to_string(),
            locked: self.locked,
        });
    }
}
