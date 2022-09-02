mod args;
mod reader;

use tpe::{
    input::InputEvent,
    events::TransactionEvent,
    Result,
};

fn main() -> Result {
    let input_path = args::parse_input_arg()?;
    let mut rdr = reader::read_csv(input_path)?;

    let mut transaction_service = tpe::build_transaction_service();

    for record in rdr.deserialize() {
        let event: InputEvent = record?;
        let event: TransactionEvent = event.parse()?;

        transaction_service.process_event(event)?;
    }

    let account_service = transaction_service.take();
    let report = account_service.build_report();

    println!("{:?}", report);

    return Ok(());
}
