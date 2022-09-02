mod charge_back_event;
mod deposit_event;
mod dispute_event;
mod resolve_event;
mod withdrawal_event;

use charge_back_event::ChargeBackEvent;
use deposit_event::DepositEvent;
use dispute_event::DisputeEvent;
use resolve_event::ResolveEvent;
use withdrawal_event::WithdrawalEvent;

pub enum TransactionEvent {
    Deposit(DepositEvent),
    Withdrawal(WithdrawalEvent),
    Dispute(DisputeEvent),
    Resolve(ResolveEvent),
    ChargeBack(ChargeBackEvent),
}

