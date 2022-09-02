mod charge_back_event;
mod deposit_event;
mod dispute_event;
mod resolve_event;
mod withdrawal_event;

pub use charge_back_event::ChargeBackEvent;
pub use deposit_event::DepositEvent;
pub use dispute_event::DisputeEvent;
pub use resolve_event::ResolveEvent;
pub use withdrawal_event::WithdrawalEvent;

/// Note: Deserializing TransactionEvent with serde would require using internal tags
/// which do not work with csv: https://github.com/BurntSushi/rust-csv/issues/211

/// Typed transaction event, forcing correct handling through the type-system
#[derive(Debug)]
pub enum TransactionEvent {
    Deposit(DepositEvent),
    Withdrawal(WithdrawalEvent),
    Dispute(DisputeEvent),
    Resolve(ResolveEvent),
    ChargeBack(ChargeBackEvent),
}

