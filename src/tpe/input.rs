use crate::ids::{ClientId, TransactionId};
use crate::Money;
use crate::Result;

use crate::{Transaction, TransactionType};

use serde::Deserialize;

use thiserror::Error;

/// Represents an input event that a string would deserialize into
#[derive(Deserialize, Debug, Clone)]
pub struct InputEvent {
    #[serde(rename = "type")]
    pub typ: InputEventType,

    pub client: u16,
    pub tx: u32,
    pub amount: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
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
    #[error("Error parsing input event: amount value missing from deposit: {0:?}")]
    NoDepositAmount(InputEvent),

    #[error("Error parsing input event: amount value missing from withdrawal: {0:?}")]
    NoWithdrawalAmount(InputEvent),

    #[error("Error parsing input event: negative amount values not supported: {0:?}")]
    NegativeAmount(InputEvent),
}

impl InputEvent {
    pub fn parse_transaction(self) -> Result<Transaction> {
        let tx = match self.typ {
            InputEventType::Deposit => {
                let amount = self
                    .clone()
                    .amount
                    .ok_or_else(|| InputParseError::NoDepositAmount(self.clone()))?;
                let amount = Money::parse(amount)?;

                if amount.0 < 0 {
                    Err(InputParseError::NegativeAmount(self.clone()))?;
                }

                Transaction {
                    id: TransactionId(self.tx),
                    client_id: ClientId(self.client),
                    tx_type: TransactionType::Deposit { amount },
                    invalid: false,
                }
            }
            InputEventType::Withdrawal => {
                let amount = self
                    .amount
                    .clone()
                    .ok_or_else(|| InputParseError::NoWithdrawalAmount(self.clone()))?;
                let amount = Money::parse(amount)?;

                if amount.0 < 0 {
                    Err(InputParseError::NegativeAmount(self.clone()))?;
                }

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

        Ok(tx)
    }
}
