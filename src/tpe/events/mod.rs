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

pub enum TransactionEvent {
    Deposit(DepositEvent),
    Withdrawal(WithdrawalEvent),
    Dispute(DisputeEvent),
    Resolve(ResolveEvent),
    ChargeBack(ChargeBackEvent),
}

