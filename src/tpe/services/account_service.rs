use crate::ids::ClientId;
use crate::models::{
    Account, ChargedBackTransaction, DisputedTransaction, TransactionType, ValidTransaction,
};
use crate::Result;

use std::collections::HashMap;

use thiserror::Error;

pub type AccountDataStore = HashMap<ClientId, Account>;

#[derive(Error, Debug)]
pub enum AccountServiceError {
    #[error("Invalid account snapshot: {0}")]
    InvalidSnapshot(String),

    #[error("Account not found: {0}")]
    AccountNotFound(ClientId),

    #[error("Account locked, cannot process transactions")]
    AccountLocked(ClientId),
}

pub struct AccountService {
    repository: AccountDataStore,
}

impl AccountService {
    pub fn new() -> Self {
        return Self {
            repository: AccountDataStore::new(),
        };
    }

    pub fn process_valid_transaction(
        &mut self,
        client_id: ClientId,
        transaction: &ValidTransaction,
    ) -> Result {
        let account = self.find_or_create(client_id);

        if account.snapshot.locked {
            Err(AccountServiceError::AccountLocked(client_id))?
        }

        match transaction.transaction_type {
            TransactionType::Deposit => {
                account.snapshot.available.add(&transaction.amount)?;
            }
            TransactionType::Withdrawal => {
                account.snapshot.available.sub(&transaction.amount)?;
            }
        }

        return Ok(());
    }

    pub fn process_dispute_transaction(
        &mut self,
        client_id: &ClientId,
        transaction: &DisputedTransaction,
    ) -> Result {
        let account = self
            .find_mut(client_id)
            .ok_or_else(|| AccountServiceError::AccountNotFound(*client_id))?;

        if account.snapshot.locked {
            Err(AccountServiceError::AccountLocked(*client_id))?
        }

        match transaction.transaction_type {
            TransactionType::Deposit => {
                account.snapshot.available.sub(&transaction.amount)?;
                account.snapshot.held.add(&transaction.amount)?;
            }
            TransactionType::Withdrawal => {
                account.snapshot.held.add(&transaction.amount)?;
            }
        }

        return Ok(());
    }

    pub fn process_resolve_transaction(
        &mut self,
        client_id: &ClientId,
        transaction: &ValidTransaction,
    ) -> Result {
        let account = self
            .find_mut(client_id)
            .ok_or_else(|| AccountServiceError::AccountNotFound(*client_id))?;

        if account.snapshot.locked {
            Err(AccountServiceError::AccountLocked(*client_id))?
        }

        match transaction.transaction_type {
            TransactionType::Deposit => {
                account.snapshot.held.sub(&transaction.amount)?;
                account.snapshot.available.add(&transaction.amount)?;
            }
            TransactionType::Withdrawal => {
                account.snapshot.held.sub(&transaction.amount)?;
            }
        }

        return Ok(());
    }

    pub fn process_charge_back_transaction(
        &mut self,
        client_id: &ClientId,
        transaction: &ChargedBackTransaction,
    ) -> Result {
        let account = self
            .find_mut(client_id)
            .ok_or_else(|| AccountServiceError::AccountNotFound(*client_id))?;

        if account.snapshot.locked {
            Err(AccountServiceError::AccountLocked(*client_id))?
        }

        match transaction.transaction_type {
            TransactionType::Deposit => {
                account.snapshot.held.sub(&transaction.amount)?;
            }
            TransactionType::Withdrawal => {
                account.snapshot.held.sub(&transaction.amount)?;
                account.snapshot.available.add(&transaction.amount)?;
            }
        }

        account.snapshot.locked = true;

        return Ok(());
    }

    pub fn find_mut(&mut self, client_id: &ClientId) -> Option<&mut Account> {
        return self.repository.get_mut(&client_id);
    }

    fn find_or_create(&mut self, client_id: ClientId) -> &mut Account {
        if self.repository.get(&client_id).is_none() {
            let account = Account::new(client_id);
            self.repository.insert(client_id, account);
        }

        return self.repository.get_mut(&client_id).unwrap();
    }
}
