use crate::events::{TransactionEvent, DepositEvent, WithdrawalEvent, DisputeEvent, ResolveEvent, ChargeBackEvent};
use crate::ids::{TransactionId, ClientId};
use crate::Money;
use crate::Result;

use serde::Deserialize;

use thiserror::Error;

/// Represents an input event that a string would deserialize into
#[derive(Deserialize, Debug)]
pub struct InputEvent {
    #[serde(rename = "type")]
    pub typ: InputEventType,

    pub client: u16,
    pub tx: u32,
    pub amount: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum InputEventType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback
}

#[derive(Error, Debug)]
pub enum InputParseError {
    #[error("No deposit amount")]
    NoDepositAmount,

    #[error("No deposit amount")]
    NoWithdrawalAmount,
}

impl InputEvent {
    /// Parse an InputEvent as a TransactionEvent for use within the library
    pub fn parse(self) -> Result<TransactionEvent> {
        let event = match self.typ {
            InputEventType::Deposit => {
                let amount = self.amount.ok_or_else(|| InputParseError::NoDepositAmount)?;
                let amount = Money::parse(amount)?;

                TransactionEvent::Deposit(DepositEvent {
                    transaction_id: TransactionId(self.tx),
                    client_id: ClientId(self.client),
                    amount,
                })
            },
            InputEventType::Withdrawal => {
                let amount = self.amount.ok_or_else(|| InputParseError::NoWithdrawalAmount)?;
                let amount = Money::parse(amount)?;

                TransactionEvent::Withdrawal(WithdrawalEvent {
                    transaction_id: TransactionId(self.tx),
                    client_id: ClientId(self.client),
                    amount,
                })
            },
            InputEventType::Dispute => TransactionEvent::Dispute(DisputeEvent {
                transaction_id: TransactionId(self.tx),
                client_id: ClientId(self.client),
            }),
            InputEventType::Resolve => TransactionEvent::Resolve(ResolveEvent {
                transaction_id: TransactionId(self.tx),
                client_id: ClientId(self.client),
            }),
            InputEventType::Chargeback => TransactionEvent::ChargeBack(ChargeBackEvent {
                transaction_id: TransactionId(self.tx),
                client_id: ClientId(self.client),
            }),
        };

        return Ok(event);
    }
}
