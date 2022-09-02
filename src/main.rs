use tpe::Result;

fn main() -> Result {
    let mut transaction_service = tpe::build_transaction_service();

    let event = tpe::events::TransactionEvent::Deposit(tpe::events::DepositEvent {
        transaction_id: tpe::ids::TransactionId(1),
        client_id: tpe::ids::ClientId(1),
        amount: tpe::Money(1),
    });

    let mut rdr = csv::Reader::from_reader("\
type,client,tx,amount
deposit,1,1,1.
".as_bytes());

    for record in rdr.deserialize() {
        let record: tpe::input::InputEvent = record?;
        println!("{:?}", record);

        let event = record.parse()?;
        println!("{:?}", event);
    }

    transaction_service.process_event(event)?;

    return Ok(());
}

