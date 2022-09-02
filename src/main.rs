use tpe::Result;

fn main() -> Result {
    let mut transaction_service = tpe::build_transaction_service();

    let event = tpe::events::TransactionEvent::Deposit(tpe::events::DepositEvent {
        transaction_id: tpe::ids::TransactionId(1),
        client_id: tpe::ids::ClientId(1),
        amount: tpe::Money(1),
    });

    transaction_service.process_event(event)?;

    return Ok(());
}

