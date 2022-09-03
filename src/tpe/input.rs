use crate::ids::{ClientId, TransactionId};
use crate::Money;
use crate::Result;

use crate::{Transaction, TransactionType};

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
    Chargeback,
}

#[derive(Error, Debug)]
pub enum InputParseError {
    #[error("No deposit amount")]
    NoDepositAmount,

    #[error("No deposit amount")]
    NoWithdrawalAmount,
}

impl InputEvent {
    pub fn parse_transaction(self) -> Result<Transaction> {
        let tx = match self.typ {
            InputEventType::Deposit => {
                let amount = self.amount.ok_or(InputParseError::NoDepositAmount)?;
                let amount = Money::parse(amount)?;

                Transaction {
                    id: TransactionId(self.tx),
                    client_id: ClientId(self.client),
                    tx_type: TransactionType::Deposit { amount },
                    invalid: false,
                }
            }
            InputEventType::Withdrawal => {
                let amount = self.amount.ok_or(InputParseError::NoDepositAmount)?;
                let amount = Money::parse(amount)?;

                Transaction {
                    id: TransactionId(self.tx),
                    client_id: ClientId(self.client),
                    tx_type: TransactionType::Withdrawal { amount },
                    invalid: false,
                }
            }
            InputEventType::Dispute => Transaction {
                id: TransactionId(self.tx),
                client_id: ClientId(self.client),
                tx_type: TransactionType::Dispute,
                invalid: false,
            },
            InputEventType::Resolve => Transaction {
                id: TransactionId(self.tx),
                client_id: ClientId(self.client),
                tx_type: TransactionType::Resolve,
                invalid: false,
            },
            InputEventType::Chargeback => Transaction {
                id: TransactionId(self.tx),
                client_id: ClientId(self.client),
                tx_type: TransactionType::ChargeBack,
                invalid: false,
            },
        };

        return Ok(tx);
    }
}
