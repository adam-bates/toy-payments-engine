use crate::ids::ClientId;
use crate::models::{
    Account, ChargedBackTransaction, DisputedTransaction, Transaction, TransactionType,
    ValidTransaction,
};
use crate::AccountReport;
use crate::Result;

use std::collections::HashMap;

use thiserror::Error;

pub type AccountDataStore = HashMap<ClientId, Account>;

#[derive(Error, Debug)]
pub enum AccountServiceError {
    #[error("Account not found: {0}")]
    AccountNotFound(ClientId),

    #[error("Account locked, cannot process transactions")]
    AccountLocked(ClientId),

    #[error("Invalid withdrawal attempt: {0}")]
    InvalidWithdrawal(String),
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

    pub fn build_report(&self) -> Result<Vec<AccountReport>> {
        let mut report = vec![];

        log::debug!("Building report...");

        for account in self.repository.values() {
            log::debug!("Building report for account: {account:?}");

            let mut total = account.snapshot.available.clone();
            total.add(&account.snapshot.held)?;
            log::debug!(
                "Calculated total as {} + {} = {}",
                account.snapshot.available,
                account.snapshot.held,
                total
            );

            report.push(AccountReport {
                client: account.client_id.to_string(),
                available: account.snapshot.available.to_string(),
                held: account.snapshot.held.to_string(),
                total: total.to_string(),
                locked: account.snapshot.locked,
            });
        }

        log::debug!("Succssfully built report!");

        return Ok(report);
    }

    pub fn process_valid_transaction(
        &mut self,
        client_id: ClientId,
        transaction: ValidTransaction,
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
                if account.snapshot.available.0 < transaction.amount.0 {
                    Err(AccountServiceError::InvalidWithdrawal(format!(
                        "Cannot withdraw {} from client {} when available amount is {}",
                        transaction.amount, account.client_id, account.snapshot.available
                    )))?
                }
                account.snapshot.available.sub(&transaction.amount)?;
            }
        }

        account.snapshot.from = Some(transaction.id);

        account.transactions.push(Transaction::Valid(transaction));

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

#[cfg(test)]
mod tests {
    use crate::{ids::TransactionId, models::{new_transaction, Snapshot}, Money};

    use super::*;

    const SOME_TRANSACTION_ID: TransactionId = TransactionId(123);
    const OTHER_TRANSACTION_ID: TransactionId = TransactionId(321);

    const SOME_CLIENT_ID: ClientId = ClientId(40);
    const OTHER_CLIENT_ID: ClientId = ClientId(41);

    const SOME_AMOUNT: Money = Money(555444);
    const OTHER_AMOUNT: Money = Money(1000);

    #[test]
    fn process_valid_deposit() {
        let mut account_service = AccountService::new();

        let transaction1 =
            new_transaction(SOME_TRANSACTION_ID, TransactionType::Deposit, SOME_AMOUNT);

        let transaction2 =
            new_transaction(OTHER_TRANSACTION_ID, TransactionType::Deposit, OTHER_AMOUNT);

        let res = account_service.process_valid_transaction(SOME_CLIENT_ID, transaction1.clone());
        assert_eq!(res.is_ok(), true);

        let res = account_service.process_valid_transaction(OTHER_CLIENT_ID, transaction2.clone());
        assert_eq!(res.is_ok(), true);

        assert_eq!(account_service.repository.len(), 2);

        assert_eq!(
            account_service
                .repository
                .get(&SOME_CLIENT_ID)
                .unwrap()
                .transactions
                .get_all(),
            vec![&Transaction::Valid(transaction1)]
        );
        assert_eq!(
            account_service
                .repository
                .get(&SOME_CLIENT_ID)
                .unwrap()
                .snapshot,
            Snapshot {
                from: Some(SOME_TRANSACTION_ID),
                available: SOME_AMOUNT,
                held: Money(0),
                locked: false,
            }
        );
        
        assert_eq!(
            account_service
                .repository
                .get(&OTHER_CLIENT_ID)
                .unwrap()
                .transactions
                .get_all(),
            vec![&Transaction::Valid(transaction2)]
        );
        assert_eq!(
            account_service
                .repository
                .get(&OTHER_CLIENT_ID)
                .unwrap()
                .snapshot,
            Snapshot {
                from: Some(OTHER_TRANSACTION_ID),
                available: OTHER_AMOUNT,
                held: Money(0),
                locked: false,
            }
        );
    }

    #[test]
    fn process_valid_withdrawal() {
        let mut account_service = AccountService::new();

        let transaction1 =
            new_transaction(SOME_TRANSACTION_ID, TransactionType::Deposit, SOME_AMOUNT);
        account_service.process_valid_transaction(SOME_CLIENT_ID, transaction1.clone()).unwrap();

        let transaction2 =
            new_transaction(OTHER_TRANSACTION_ID, TransactionType::Withdrawal, SOME_AMOUNT);

        let res = account_service.process_valid_transaction(SOME_CLIENT_ID, transaction2.clone());
        assert_eq!(res.is_ok(), true);

        assert_eq!(account_service.repository.len(), 1);
        assert_eq!(
            account_service
                .repository
                .get(&SOME_CLIENT_ID)
                .unwrap()
                .transactions
                .get_all(),
            vec![&Transaction::Valid(transaction1), &Transaction::Valid(transaction2)]
        );
        assert_eq!(
            account_service
                .repository
                .get(&SOME_CLIENT_ID)
                .unwrap()
                .snapshot,
            Snapshot {
                from: Some(OTHER_TRANSACTION_ID),
                available: Money(0),
                held: Money(0),
                locked: false,
            }
        );
    }

    #[test]
    fn process_dispute_deposit() {
        let mut account_service = AccountService::new();

        let transaction =
            new_transaction(SOME_TRANSACTION_ID, TransactionType::Deposit, SOME_AMOUNT);
        account_service.process_valid_transaction(SOME_CLIENT_ID, transaction.clone()).unwrap();

        let transaction = transaction.dispute();

        let res = account_service.process_dispute_transaction(&SOME_CLIENT_ID, &transaction);
        assert_eq!(res.is_ok(), true);

        assert_eq!(account_service.repository.len(), 1);
        assert_eq!(
            account_service
                .repository
                .get(&SOME_CLIENT_ID)
                .unwrap()
                .snapshot,
            Snapshot {
                from: Some(SOME_TRANSACTION_ID),
                available: Money(0),
                held: SOME_AMOUNT,
                locked: false,
            }
        );
    }

    #[test]
    fn process_dispute_withdrawal() {
        let mut account_service = AccountService::new();

        let transaction1 =
            new_transaction(SOME_TRANSACTION_ID, TransactionType::Deposit, SOME_AMOUNT);
        account_service.process_valid_transaction(SOME_CLIENT_ID, transaction1.clone()).unwrap();

        let transaction2 =
            new_transaction(OTHER_TRANSACTION_ID, TransactionType::Withdrawal, SOME_AMOUNT);
        account_service.process_valid_transaction(SOME_CLIENT_ID, transaction2.clone()).unwrap();

        let transaction2 = transaction2.dispute();

        let res = account_service.process_dispute_transaction(&SOME_CLIENT_ID, &transaction2);
        assert_eq!(res.is_ok(), true);

        assert_eq!(account_service.repository.len(), 1);
        assert_eq!(
            account_service
                .repository
                .get(&SOME_CLIENT_ID)
                .unwrap()
                .snapshot,
            Snapshot {
                from: Some(OTHER_TRANSACTION_ID),
                available: Money(0),
                held: SOME_AMOUNT,
                locked: false,
            }
        );
    }

    #[test]
    fn process_resolve_deposit() {
        let mut account_service = AccountService::new();

        let transaction =
            new_transaction(SOME_TRANSACTION_ID, TransactionType::Deposit, SOME_AMOUNT);
        account_service.process_valid_transaction(SOME_CLIENT_ID, transaction.clone()).unwrap();

        let transaction = transaction.dispute();
        account_service.process_dispute_transaction(&SOME_CLIENT_ID, &transaction).unwrap();

        let transaction = transaction.resolve();

        let res = account_service.process_resolve_transaction(&SOME_CLIENT_ID, &transaction);
        assert_eq!(res.is_ok(), true);

        assert_eq!(account_service.repository.len(), 1);
        assert_eq!(
            account_service
                .repository
                .get(&SOME_CLIENT_ID)
                .unwrap()
                .snapshot,
            Snapshot {
                from: Some(SOME_TRANSACTION_ID),
                available: SOME_AMOUNT,
                held: Money(0),
                locked: false,
            }
        );
    }

    #[test]
    fn process_resolve_withdrawal() {
        let mut account_service = AccountService::new();

        let transaction1 =
            new_transaction(SOME_TRANSACTION_ID, TransactionType::Deposit, SOME_AMOUNT);
        account_service.process_valid_transaction(SOME_CLIENT_ID, transaction1.clone()).unwrap();

        let transaction2 =
            new_transaction(OTHER_TRANSACTION_ID, TransactionType::Withdrawal, SOME_AMOUNT);
        account_service.process_valid_transaction(SOME_CLIENT_ID, transaction2.clone()).unwrap();

        let transaction2 = transaction2.dispute();
        account_service.process_dispute_transaction(&SOME_CLIENT_ID, &transaction2).unwrap();

        let transaction2 = transaction2.resolve();

        let res = account_service.process_resolve_transaction(&SOME_CLIENT_ID, &transaction2);
        assert_eq!(res.is_ok(), true);

        assert_eq!(account_service.repository.len(), 1);
        assert_eq!(
            account_service
                .repository
                .get(&SOME_CLIENT_ID)
                .unwrap()
                .snapshot,
            Snapshot {
                from: Some(OTHER_TRANSACTION_ID),
                available: Money(0),
                held: Money(0),
                locked: false,
            }
        );
    }

    #[test]
    fn process_charge_back_deposit() {
        let mut account_service = AccountService::new();

        let transaction =
            new_transaction(SOME_TRANSACTION_ID, TransactionType::Deposit, SOME_AMOUNT);
        account_service.process_valid_transaction(SOME_CLIENT_ID, transaction.clone()).unwrap();

        let transaction = transaction.dispute();
        account_service.process_dispute_transaction(&SOME_CLIENT_ID, &transaction).unwrap();

        let transaction = transaction.charge_back();

        let res = account_service.process_charge_back_transaction(&SOME_CLIENT_ID, &transaction);
        assert_eq!(res.is_ok(), true);

        assert_eq!(account_service.repository.len(), 1);
        assert_eq!(
            account_service
                .repository
                .get(&SOME_CLIENT_ID)
                .unwrap()
                .snapshot,
            Snapshot {
                from: Some(SOME_TRANSACTION_ID),
                available: Money(0),
                held: Money(0),
                locked: true,
            }
        );
    }

    #[test]
    fn process_charge_back_withdrawal() {
        let mut account_service = AccountService::new();

        let transaction1 =
            new_transaction(SOME_TRANSACTION_ID, TransactionType::Deposit, SOME_AMOUNT);
        account_service.process_valid_transaction(SOME_CLIENT_ID, transaction1.clone()).unwrap();

        let transaction2 =
            new_transaction(OTHER_TRANSACTION_ID, TransactionType::Withdrawal, SOME_AMOUNT);
        account_service.process_valid_transaction(SOME_CLIENT_ID, transaction2.clone()).unwrap();

        let transaction2 = transaction2.dispute();
        account_service.process_dispute_transaction(&SOME_CLIENT_ID, &transaction2).unwrap();

        let transaction2 = transaction2.charge_back();

        let res = account_service.process_charge_back_transaction(&SOME_CLIENT_ID, &transaction2);
        assert_eq!(res.is_ok(), true);

        assert_eq!(account_service.repository.len(), 1);
        assert_eq!(
            account_service
                .repository
                .get(&SOME_CLIENT_ID)
                .unwrap()
                .snapshot,
            Snapshot {
                from: Some(OTHER_TRANSACTION_ID),
                available: SOME_AMOUNT,
                held: Money(0),
                locked: true,
            }
        );
    }

    #[test]
    fn fail_to_process_invalid_withdrawal() {
        let mut account_service = AccountService::new();

        let transaction1 =
            new_transaction(SOME_TRANSACTION_ID, TransactionType::Deposit, SOME_AMOUNT);
        account_service.process_valid_transaction(SOME_CLIENT_ID, transaction1.clone()).unwrap();

        let mut invalid_amount = SOME_AMOUNT.clone();
        invalid_amount.add(&OTHER_AMOUNT).unwrap();

        let transaction2 =
            new_transaction(OTHER_TRANSACTION_ID, TransactionType::Withdrawal, invalid_amount);

        let res = account_service.process_valid_transaction(SOME_CLIENT_ID, transaction2.clone());
        assert_eq!(res.is_ok(), false);

        let e = res.err().unwrap();

        match e.downcast_ref::<AccountServiceError>() {
            Some(e) => match e {
                AccountServiceError::InvalidWithdrawal(_) => assert_eq!(true, true),
                _ => panic!("Invalid: {e}"),
            },
            _ => panic!("Invalid: {e}"),
        }

        assert_eq!(account_service.repository.len(), 1);
        assert_eq!(
            account_service
                .repository
                .get(&SOME_CLIENT_ID)
                .unwrap()
                .transactions
                .get_all(),
            vec![&Transaction::Valid(transaction1)]
        );
        assert_eq!(
            account_service
                .repository
                .get(&SOME_CLIENT_ID)
                .unwrap()
                .snapshot,
            Snapshot {
                from: Some(SOME_TRANSACTION_ID),
                available: SOME_AMOUNT,
                held: Money(0),
                locked: false,
            }
        );
    }

    #[test]
    fn fail_to_process_invalid_client_id() {
        let mut account_service = AccountService::new();

        let transaction =
            new_transaction(SOME_TRANSACTION_ID, TransactionType::Deposit, SOME_AMOUNT);
        account_service.process_valid_transaction(SOME_CLIENT_ID, transaction.clone()).unwrap();

        let transaction = transaction.dispute();

        let res = account_service.process_dispute_transaction(&OTHER_CLIENT_ID, &transaction);
        assert_eq!(res.is_ok(), false);

        let e = res.err().unwrap();

        match e.downcast_ref::<AccountServiceError>() {
            Some(e) => match e {
                AccountServiceError::AccountNotFound(_) => assert_eq!(true, true),
                _ => panic!("Invalid: {e}"),
            },
            _ => panic!("Invalid: {e}"),
        }

        let resolved = transaction.clone().resolve();

        let res = account_service.process_resolve_transaction(&OTHER_CLIENT_ID, &resolved);
        assert_eq!(res.is_ok(), false);

        let e = res.err().unwrap();

        match e.downcast_ref::<AccountServiceError>() {
            Some(e) => match e {
                AccountServiceError::AccountNotFound(_) => assert_eq!(true, true),
                _ => panic!("Invalid: {e}"),
            },
            _ => panic!("Invalid: {e}"),
        }

        let charged_back = transaction.clone().charge_back();

        let res = account_service.process_charge_back_transaction(&OTHER_CLIENT_ID, &charged_back);
        assert_eq!(res.is_ok(), false);

        let e = res.err().unwrap();

        match e.downcast_ref::<AccountServiceError>() {
            Some(e) => match e {
                AccountServiceError::AccountNotFound(_) => assert_eq!(true, true),
                _ => panic!("Invalid: {e}"),
            },
            _ => panic!("Invalid: {e}"),
        }

        assert_eq!(account_service.repository.len(), 1);
        assert_eq!(
            account_service
                .repository
                .get(&SOME_CLIENT_ID)
                .unwrap()
                .snapshot,
            Snapshot {
                from: Some(SOME_TRANSACTION_ID),
                available: SOME_AMOUNT,
                held: Money(0),
                locked: false,
            }
        );
    }

    #[test]
    fn fail_to_process_on_locked_account() {
        let mut account_service = AccountService::new();

        let transaction1 =
            new_transaction(SOME_TRANSACTION_ID, TransactionType::Deposit, SOME_AMOUNT);
        account_service.process_valid_transaction(SOME_CLIENT_ID, transaction1.clone()).unwrap();

        let transaction1 = transaction1.dispute();
        account_service.process_dispute_transaction(&SOME_CLIENT_ID, &transaction1).unwrap();

        let transaction1 = transaction1.charge_back();
        account_service.process_charge_back_transaction(&SOME_CLIENT_ID, &transaction1).unwrap();

        let transaction2 = new_transaction(OTHER_TRANSACTION_ID, TransactionType::Deposit, SOME_AMOUNT);

        let res = account_service.process_valid_transaction(SOME_CLIENT_ID, transaction2.clone());
        assert_eq!(res.is_ok(), false);

        let e = res.err().unwrap();

        match e.downcast_ref::<AccountServiceError>() {
            Some(e) => match e {
                AccountServiceError::AccountLocked(_) => assert_eq!(true, true),
                _ => panic!("Invalid: {e}"),
            },
            _ => panic!("Invalid: {e}"),
        }

        let transaction2 = transaction2.dispute();

        let res = account_service.process_dispute_transaction(&SOME_CLIENT_ID, &transaction2);
        assert_eq!(res.is_ok(), false);

        let e = res.err().unwrap();

        match e.downcast_ref::<AccountServiceError>() {
            Some(e) => match e {
                AccountServiceError::AccountLocked(_) => assert_eq!(true, true),
                _ => panic!("Invalid: {e}"),
            },
            _ => panic!("Invalid: {e}"),
        }


        let resolved = transaction2.clone().resolve();

        let res = account_service.process_resolve_transaction(&SOME_CLIENT_ID, &resolved);
        assert_eq!(res.is_ok(), false);

        let e = res.err().unwrap();

        match e.downcast_ref::<AccountServiceError>() {
            Some(e) => match e {
                AccountServiceError::AccountLocked(_) => assert_eq!(true, true),
                _ => panic!("Invalid: {e}"),
            },
            _ => panic!("Invalid: {e}"),
        }

        let charged_back = transaction2.clone().charge_back();

        let res = account_service.process_charge_back_transaction(&SOME_CLIENT_ID, &charged_back);
        assert_eq!(res.is_ok(), false);

        let e = res.err().unwrap();

        match e.downcast_ref::<AccountServiceError>() {
            Some(e) => match e {
                AccountServiceError::AccountLocked(_) => assert_eq!(true, true),
                _ => panic!("Invalid: {e}"),
            },
            _ => panic!("Invalid: {e}"),
        }

        assert_eq!(account_service.repository.len(), 1);
        assert_eq!(
            account_service
                .repository
                .get(&SOME_CLIENT_ID)
                .unwrap()
                .snapshot,
            Snapshot {
                from: Some(SOME_TRANSACTION_ID),
                available: Money(0),
                held: Money(0),
                locked: true,
            }
        );
    }
}
