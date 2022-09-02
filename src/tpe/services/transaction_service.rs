use super::AccountService;

use crate::events::{
    ChargeBackEvent, DepositEvent, DisputeEvent, ResolveEvent, TransactionEvent, WithdrawalEvent,
};
use crate::models::{
    new_transaction, ChargedBackTransaction, DisputedTransaction, Transaction, TransactionType,
    ValidTransaction,
};
use crate::Result;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransactionServiceError {
    #[error("Invalid transaction event: {0}")]
    InvalidEvent(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub struct TransactionService {
    account_service: AccountService,
}

impl TransactionService {
    pub fn new(account_service: AccountService) -> Self {
        return Self {
            account_service,
        };
    }

    pub fn process_event(&mut self, event: TransactionEvent) -> Result {
        log::debug!("Processing transaction event: {event:?}");

        match event {
            TransactionEvent::Deposit(event) => self.process_deposit_event(event)?,

            TransactionEvent::Withdrawal(event) => self.process_withdrawal_event(event)?,

            TransactionEvent::Dispute(event) => self.process_dispute_event(event)?,

            TransactionEvent::Resolve(event) => self.process_resolve_event(event)?,

            TransactionEvent::ChargeBack(event) => self.process_charge_back_event(event)?,
        }

        return Ok(());
    }

    pub fn take(self) -> AccountService {
        log::debug!("Destructuring TransactionService");
        return self.account_service;
    }

    fn process_deposit_event(&mut self, event: DepositEvent) -> Result {
        let transaction =
            new_transaction(event.transaction_id, TransactionType::Deposit, event.amount);

        log::debug!("Successfully created new transaction: {transaction:?}");

        self.account_service
            .process_valid_transaction(event.client_id, &transaction)?;

        return Ok(());
    }

    fn process_withdrawal_event(&mut self, event: WithdrawalEvent) -> Result {
        let transaction = new_transaction(
            event.transaction_id,
            TransactionType::Withdrawal,
            event.amount,
        );

        log::debug!("Successfully created new transaction: {transaction:?}");

        self.account_service
            .process_valid_transaction(event.client_id, &transaction)?;

        return Ok(());
    }

    fn process_dispute_event(&mut self, event: DisputeEvent) -> Result {
        let account = self
            .account_service
            .find_mut(&event.client_id)
            .ok_or_else(|| {
                TransactionServiceError::InvalidEvent(format!(
                    "Dispute event contains an invalid client_id: {}",
                    event.client_id
                ))
            })?;

        log::debug!("Found account: {account:?}");

        let transaction = account
            .transactions
            .replace(event.transaction_id, |transaction| match transaction {
                Transaction::Valid(transaction) => {
                    return Ok(Transaction::Disputed(transaction.dispute()))
                }
                _ => Err(TransactionServiceError::InvalidEvent(format!(
                    "Dispute event on a non-valid transaction: {:?}",
                    transaction
                )))?,
            })?
            .ok_or_else(|| {
                TransactionServiceError::InvalidEvent(format!(
                    "Dispute event contains an invalid transaction_id: {} for client {}",
                    event.transaction_id,
                    event.client_id,
                ))
            })?;

        log::debug!("Updated transaction: {transaction:?}");

        let transaction: DisputedTransaction =
            if let Transaction::Disputed(transaction) = transaction {
                transaction.clone()
            } else {
                let msg = format!("Impossible state encountered: transaction was not Disputed after successful mapping: {:?}", transaction);
                log::error!("{}", msg);
                Err(TransactionServiceError::Unknown(msg))?
            };

        self.account_service
            .process_dispute_transaction(&event.client_id, &transaction)?;

        return Ok(());
    }

    fn process_resolve_event(&mut self, event: ResolveEvent) -> Result {
        let account = self
            .account_service
            .find_mut(&event.client_id)
            .ok_or_else(|| {
                TransactionServiceError::InvalidEvent(format!(
                    "Resolve event contains an invalid client_id: {}",
                    event.client_id
                ))
            })?;

        log::debug!("Found account: {account:?}");

        let transaction = account
            .transactions
            .replace(event.transaction_id, |transaction| match transaction {
                Transaction::Disputed(transaction) => {
                    return Ok(Transaction::Valid(transaction.resolve()))
                }
                _ => Err(TransactionServiceError::InvalidEvent(format!(
                    "Resolve event on a non-disputed transaction: {:?}",
                    transaction
                )))?,
            })?
            .ok_or_else(|| {
                TransactionServiceError::InvalidEvent(format!(
                    "Resolve event contains an invalid transaction_id: {}",
                    event.transaction_id
                ))
            })?;

        log::debug!("Updated transaction: {transaction:?}");

        let transaction: ValidTransaction = if let Transaction::Valid(transaction) = transaction {
            transaction.clone()
        } else {
            let msg = format!("Impossible state encountered: transaction was not Valid after successful mapping: {:?}", transaction);
            log::error!("{}", msg);
            Err(TransactionServiceError::Unknown(msg))?
        };

        self.account_service
            .process_resolve_transaction(&event.client_id, &transaction)?;

        return Ok(());
    }

    fn process_charge_back_event(&mut self, event: ChargeBackEvent) -> Result {
        let account = self
            .account_service
            .find_mut(&event.client_id)
            .ok_or_else(|| {
                TransactionServiceError::InvalidEvent(format!(
                    "ChargeBack event contains an invalid client_id: {}",
                    event.client_id
                ))
            })?;

        log::debug!("Found account: {account:?}");

        let transaction = account
            .transactions
            .replace(event.transaction_id, |transaction| match transaction {
                Transaction::Disputed(transaction) => {
                    return Ok(Transaction::ChargedBack(transaction.charge_back()))
                }
                _ => Err(TransactionServiceError::InvalidEvent(format!(
                    "ChargeBack event on a non-disputed transaction: {:?}",
                    transaction
                )))?,
            })?
            .ok_or_else(|| {
                TransactionServiceError::InvalidEvent(format!(
                    "ChargeBack event contains an invalid transaction_id: {}",
                    event.transaction_id
                ))
            })?;

        log::debug!("Updated transaction: {transaction:?}");

        let transaction: ChargedBackTransaction =
            if let Transaction::ChargedBack(transaction) = transaction {
                transaction.clone()
            } else {
                let msg = format!("Impossible state encountered: transaction was not ChargedBack after successful mapping: {:?}", transaction);
                log::error!("{}", msg);
                Err(TransactionServiceError::Unknown(msg))?
            };

        self.account_service
            .process_charge_back_transaction(&event.client_id, &transaction)?;

        return Ok(());
    }
}

