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

/// The Transaction service is responsible for processing transaction events as transactions to an
/// account.
pub struct TransactionService {
    account_service: AccountService,
}

impl TransactionService {
    pub fn new(account_service: AccountService) -> Self {
        Self { account_service }
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

        Ok(())
    }

    pub fn take(self) -> AccountService {
        log::debug!("Destructuring TransactionService");
        self.account_service
    }

    fn process_deposit_event(&mut self, event: DepositEvent) -> Result {
        let transaction =
            new_transaction(event.transaction_id, TransactionType::Deposit, event.amount);

        log::debug!("Successfully created new transaction: {transaction:?}");

        self.account_service
            .process_valid_transaction(event.client_id, transaction)?;

        Ok(())
    }

    fn process_withdrawal_event(&mut self, event: WithdrawalEvent) -> Result {
        let transaction = new_transaction(
            event.transaction_id,
            TransactionType::Withdrawal,
            event.amount,
        );

        log::debug!("Successfully created new transaction: {transaction:?}");

        self.account_service
            .process_valid_transaction(event.client_id, transaction)?;

        Ok(())
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
                Transaction::Valid(transaction) => Ok(Transaction::Disputed(transaction.dispute())),
                _ => Err(TransactionServiceError::InvalidEvent(format!(
                    "Dispute event on a non-valid transaction: {:?}",
                    transaction
                )))?,
            })?
            .ok_or_else(|| {
                TransactionServiceError::InvalidEvent(format!(
                    "Dispute event contains an invalid transaction_id: {} for client {}",
                    event.transaction_id, event.client_id,
                ))
            })?;

        log::debug!("Updated transaction: {transaction:?}");

        let transaction: DisputedTransaction = if let Transaction::Disputed(transaction) =
            transaction
        {
            transaction.clone()
        } else {
            let msg = format!("Impossible state encountered: transaction was not Disputed after successful mapping: {:?}", transaction);
            log::error!("{}", msg);
            Err(TransactionServiceError::Unknown(msg))?
        };

        if let Err(e) = self
            .account_service
            .process_dispute_transaction(&event.client_id, &transaction)
        {
            log::debug!("Rolling back transaction change!");

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

            account
                .transactions
                .replace(event.transaction_id, |transaction| match transaction {
                    Transaction::Disputed(transaction) => {
                        Ok(Transaction::Valid(new_transaction(transaction.id, transaction.transaction_type.clone(), transaction.amount)))
                    }
                    _ => Ok(transaction),
                })?
                .ok_or_else(|| {
                    TransactionServiceError::InvalidEvent(format!(
                        "Dispute event contains an invalid transaction_id: {} for client {}",
                        event.transaction_id, event.client_id,
                    ))
                })?;

            Err(e)?
        }

        Ok(())
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
                Transaction::Disputed(transaction) => Ok(Transaction::Valid(transaction.resolve())),
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

        Ok(())
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
                    Ok(Transaction::ChargedBack(transaction.charge_back()))
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

        let transaction: ChargedBackTransaction = if let Transaction::ChargedBack(transaction) =
            transaction
        {
            transaction.clone()
        } else {
            let msg = format!("Impossible state encountered: transaction was not ChargedBack after successful mapping: {:?}", transaction);
            log::error!("{}", msg);
            Err(TransactionServiceError::Unknown(msg))?
        };

        self.account_service
            .process_charge_back_transaction(&event.client_id, &transaction)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ids::{ClientId, TransactionId},
        services::AccountServiceError,
        AccountReport, Money,
    };

    use super::*;

    const VALID_DEPOSIT_TRANSACTION_ID: TransactionId = TransactionId(11);
    const DISPUTED_DEPOSIT_TRANSACTION_ID: TransactionId = TransactionId(12);
    const RESOLVED_DEPOSIT_TRANSACTION_ID: TransactionId = TransactionId(13);
    const CHARGED_BACK_DEPOSIT_TRANSACTION_ID: TransactionId = TransactionId(14);

    const VALID_WITHDRAWAL_TRANSACTION_ID: TransactionId = TransactionId(21);
    const DISPUTED_WITHDRAWAL_TRANSACTION_ID: TransactionId = TransactionId(22);
    const RESOLVED_WITHDRAWAL_TRANSACTION_ID: TransactionId = TransactionId(23);
    const CHARGED_BACK_WITHDRAWAL_TRANSACTION_ID: TransactionId = TransactionId(24);

    const SOME_CLIENT_ID: ClientId = ClientId(456);
    const OTHER_CLIENT_ID: ClientId = ClientId(789);

    const SOME_SMALL_AMOUNT: Money = Money(1);
    const SOME_AMOUNT: Money = Money(42000);
    const SOME_LARGE_AMOUNT: Money = Money(123456000);

    #[test]
    fn process_simple_deposit() {
        let mut transaction_service = TransactionService::new(AccountService::new());

        let event = TransactionEvent::Deposit(DepositEvent {
            transaction_id: VALID_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
            amount: SOME_AMOUNT,
        });

        let res = transaction_service.process_event(event);
        assert!(res.is_ok());

        let account_service = transaction_service.take();
        assert_eq!(
            account_service.build_report().unwrap(),
            vec![AccountReport {
                client: SOME_CLIENT_ID.to_string(),
                available: SOME_AMOUNT.to_string(),
                held: Money(0).to_string(),
                total: SOME_AMOUNT.to_string(),
                locked: false,
            }]
        );
    }

    #[test]
    fn process_simple_withdrawal() {
        let mut transaction_service = TransactionService::new(AccountService::new());

        let event = TransactionEvent::Deposit(DepositEvent {
            transaction_id: VALID_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
            amount: SOME_AMOUNT,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::Withdrawal(WithdrawalEvent {
            transaction_id: VALID_WITHDRAWAL_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
            amount: SOME_AMOUNT,
        });

        let res = transaction_service.process_event(event);
        assert!(res.is_ok());

        let account_service = transaction_service.take();
        assert_eq!(
            account_service.build_report().unwrap(),
            vec![AccountReport {
                client: SOME_CLIENT_ID.to_string(),
                available: Money(0).to_string(),
                held: Money(0).to_string(),
                total: Money(0).to_string(),
                locked: false,
            }]
        );
    }

    #[test]
    fn process_deposit_dispute() {
        let mut transaction_service = TransactionService::new(AccountService::new());

        let event = TransactionEvent::Deposit(DepositEvent {
            transaction_id: DISPUTED_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
            amount: SOME_AMOUNT,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::Deposit(DepositEvent {
            transaction_id: VALID_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
            amount: SOME_SMALL_AMOUNT,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::Dispute(DisputeEvent {
            transaction_id: DISPUTED_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
        });

        let res = transaction_service.process_event(event);
        assert!(res.is_ok());

        let account_service = transaction_service.take();

        let mut expected_total = SOME_AMOUNT;
        expected_total.add(&SOME_SMALL_AMOUNT).unwrap();

        assert_eq!(
            account_service.build_report().unwrap(),
            vec![AccountReport {
                client: SOME_CLIENT_ID.to_string(),
                available: SOME_SMALL_AMOUNT.to_string(),
                held: SOME_AMOUNT.to_string(),
                total: expected_total.to_string(),
                locked: false,
            }]
        );
    }

    #[test]
    fn process_withdrawal_dispute() {
        let mut transaction_service = TransactionService::new(AccountService::new());

        let event = TransactionEvent::Deposit(DepositEvent {
            transaction_id: VALID_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
            amount: SOME_AMOUNT,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::Withdrawal(WithdrawalEvent {
            transaction_id: DISPUTED_WITHDRAWAL_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
            amount: SOME_SMALL_AMOUNT,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::Dispute(DisputeEvent {
            transaction_id: DISPUTED_WITHDRAWAL_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
        });

        let res = transaction_service.process_event(event);
        assert!(res.is_ok());

        let account_service = transaction_service.take();

        let mut expected_available = SOME_AMOUNT;
        expected_available.sub(&SOME_SMALL_AMOUNT).unwrap();

        assert_eq!(
            account_service.build_report().unwrap(),
            vec![AccountReport {
                client: SOME_CLIENT_ID.to_string(),
                available: expected_available.to_string(),
                held: SOME_SMALL_AMOUNT.to_string(),
                total: SOME_AMOUNT.to_string(),
                locked: false,
            }]
        );
    }

    #[test]
    fn process_deposit_resolve() {
        let mut transaction_service = TransactionService::new(AccountService::new());

        let event = TransactionEvent::Deposit(DepositEvent {
            transaction_id: RESOLVED_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
            amount: SOME_AMOUNT,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::Dispute(DisputeEvent {
            transaction_id: RESOLVED_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::Resolve(ResolveEvent {
            transaction_id: RESOLVED_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
        });

        let res = transaction_service.process_event(event);
        assert!(res.is_ok());

        let account_service = transaction_service.take();

        assert_eq!(
            account_service.build_report().unwrap(),
            vec![AccountReport {
                client: SOME_CLIENT_ID.to_string(),
                available: SOME_AMOUNT.to_string(),
                held: Money(0).to_string(),
                total: SOME_AMOUNT.to_string(),
                locked: false,
            }]
        );
    }

    #[test]
    fn process_withdrawal_resolve() {
        let mut transaction_service = TransactionService::new(AccountService::new());

        let event = TransactionEvent::Deposit(DepositEvent {
            transaction_id: VALID_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
            amount: SOME_AMOUNT,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::Withdrawal(WithdrawalEvent {
            transaction_id: RESOLVED_WITHDRAWAL_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
            amount: SOME_SMALL_AMOUNT,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::Dispute(DisputeEvent {
            transaction_id: RESOLVED_WITHDRAWAL_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::Resolve(ResolveEvent {
            transaction_id: RESOLVED_WITHDRAWAL_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
        });

        let res = transaction_service.process_event(event);
        assert!(res.is_ok());

        let account_service = transaction_service.take();

        let mut expected_available = SOME_AMOUNT;
        expected_available.sub(&SOME_SMALL_AMOUNT).unwrap();

        assert_eq!(
            account_service.build_report().unwrap(),
            vec![AccountReport {
                client: SOME_CLIENT_ID.to_string(),
                available: expected_available.to_string(),
                held: Money(0).to_string(),
                total: expected_available.to_string(),
                locked: false,
            }]
        );
    }

    #[test]
    fn process_deposit_charge_back() {
        let mut transaction_service = TransactionService::new(AccountService::new());

        let event = TransactionEvent::Deposit(DepositEvent {
            transaction_id: VALID_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
            amount: SOME_SMALL_AMOUNT,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::Deposit(DepositEvent {
            transaction_id: CHARGED_BACK_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
            amount: SOME_AMOUNT,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::Dispute(DisputeEvent {
            transaction_id: CHARGED_BACK_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::ChargeBack(ChargeBackEvent {
            transaction_id: CHARGED_BACK_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
        });

        let res = transaction_service.process_event(event);
        assert!(res.is_ok());

        let account_service = transaction_service.take();

        assert_eq!(
            account_service.build_report().unwrap(),
            vec![AccountReport {
                client: SOME_CLIENT_ID.to_string(),
                available: SOME_SMALL_AMOUNT.to_string(),
                held: Money(0).to_string(),
                total: SOME_SMALL_AMOUNT.to_string(),
                locked: true,
            }]
        );
    }

    #[test]
    fn process_withdrawal_charge_back() {
        let mut transaction_service = TransactionService::new(AccountService::new());

        let event = TransactionEvent::Deposit(DepositEvent {
            transaction_id: VALID_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
            amount: SOME_AMOUNT,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::Withdrawal(WithdrawalEvent {
            transaction_id: CHARGED_BACK_WITHDRAWAL_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
            amount: SOME_SMALL_AMOUNT,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::Dispute(DisputeEvent {
            transaction_id: CHARGED_BACK_WITHDRAWAL_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::ChargeBack(ChargeBackEvent {
            transaction_id: CHARGED_BACK_WITHDRAWAL_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
        });

        let res = transaction_service.process_event(event);
        assert!(res.is_ok());

        let account_service = transaction_service.take();

        assert_eq!(
            account_service.build_report().unwrap(),
            vec![AccountReport {
                client: SOME_CLIENT_ID.to_string(),
                available: SOME_AMOUNT.to_string(),
                held: Money(0).to_string(),
                total: SOME_AMOUNT.to_string(),
                locked: true,
            }]
        );
    }

    #[test]
    fn fail_to_process_dispute_unknown_client_id() {
        let mut transaction_service = TransactionService::new(AccountService::new());

        let event = TransactionEvent::Deposit(DepositEvent {
            transaction_id: DISPUTED_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
            amount: SOME_AMOUNT,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::Dispute(DisputeEvent {
            transaction_id: DISPUTED_DEPOSIT_TRANSACTION_ID,
            client_id: OTHER_CLIENT_ID,
        });

        let res = transaction_service.process_event(event);
        assert!(res.is_err());

        let e = res.err().unwrap();
        match e.downcast_ref::<TransactionServiceError>() {
            Some(e) => match e {
                TransactionServiceError::InvalidEvent(_) => {}
                _ => panic!("Invalid: {e}"),
            },
            _ => panic!("Invalid: {e}"),
        }

        let account_service = transaction_service.take();

        assert_eq!(
            account_service.build_report().unwrap(),
            vec![AccountReport {
                client: SOME_CLIENT_ID.to_string(),
                available: SOME_AMOUNT.to_string(),
                held: Money(0).to_string(),
                total: SOME_AMOUNT.to_string(),
                locked: false,
            }]
        );
    }

    #[test]
    fn fail_to_process_dispute_unknown_transaction_id() {
        let mut transaction_service = TransactionService::new(AccountService::new());

        let event = TransactionEvent::Deposit(DepositEvent {
            transaction_id: VALID_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
            amount: SOME_AMOUNT,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::Dispute(DisputeEvent {
            transaction_id: DISPUTED_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
        });

        let res = transaction_service.process_event(event);
        assert!(res.is_err());

        let e = res.err().unwrap();
        match e.downcast_ref::<TransactionServiceError>() {
            Some(e) => match e {
                TransactionServiceError::InvalidEvent(_) => {}
                _ => panic!("Invalid: {e}"),
            },
            _ => panic!("Invalid: {e}"),
        }

        let account_service = transaction_service.take();

        assert_eq!(
            account_service.build_report().unwrap(),
            vec![AccountReport {
                client: SOME_CLIENT_ID.to_string(),
                available: SOME_AMOUNT.to_string(),
                held: Money(0).to_string(),
                total: SOME_AMOUNT.to_string(),
                locked: false,
            }]
        );
    }

    #[test]
    fn fail_to_process_invalid_withdrawal() {
        let mut transaction_service = TransactionService::new(AccountService::new());

        let event = TransactionEvent::Deposit(DepositEvent {
            transaction_id: VALID_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
            amount: SOME_AMOUNT,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::Withdrawal(WithdrawalEvent {
            transaction_id: VALID_WITHDRAWAL_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
            amount: SOME_LARGE_AMOUNT,
        });

        let res = transaction_service.process_event(event);
        assert!(res.is_err());

        let e = res.err().unwrap();
        match e.downcast_ref::<AccountServiceError>() {
            Some(e) => match e {
                AccountServiceError::InvalidWithdrawal(_) => {}
                _ => panic!("Invalid: {e}"),
            },
            _ => panic!("Invalid: {e}"),
        }

        let account_service = transaction_service.take();

        assert_eq!(
            account_service.build_report().unwrap(),
            vec![AccountReport {
                client: SOME_CLIENT_ID.to_string(),
                available: SOME_AMOUNT.to_string(),
                held: Money(0).to_string(),
                total: SOME_AMOUNT.to_string(),
                locked: false,
            }]
        );
    }

    #[test]
    fn fail_to_process_invalid_dispute() {
        let mut transaction_service = TransactionService::new(AccountService::new());

        let event = TransactionEvent::Deposit(DepositEvent {
            transaction_id: DISPUTED_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
            amount: SOME_AMOUNT,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::Dispute(DisputeEvent {
            transaction_id: DISPUTED_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
        });
        transaction_service.process_event(event).unwrap();

        let event = TransactionEvent::Dispute(DisputeEvent {
            transaction_id: DISPUTED_DEPOSIT_TRANSACTION_ID,
            client_id: SOME_CLIENT_ID,
        });

        let res = transaction_service.process_event(event);
        assert!(res.is_err());

        let e = res.err().unwrap();
        match e.downcast_ref::<TransactionServiceError>() {
            Some(e) => match e {
                TransactionServiceError::InvalidEvent(_) => {}
                _ => panic!("Invalid: {e}"),
            },
            _ => panic!("Invalid: {e}"),
        }

        let account_service = transaction_service.take();

        assert_eq!(
            account_service.build_report().unwrap(),
            vec![AccountReport {
                client: SOME_CLIENT_ID.to_string(),
                available: Money(0).to_string(),
                held: SOME_AMOUNT.to_string(),
                total: SOME_AMOUNT.to_string(),
                locked: false,
            }]
        );
    }
}
