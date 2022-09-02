mod args;
mod reader;
mod writer;

use tpe::{
    input::InputEvent,
    events::TransactionEvent,
    Result, services::{TransactionService, AccountService},
};

fn main() -> Result {
    let mut transaction_service = tpe::build_transaction_service();
    process_data(&mut transaction_service)?;

    let account_service = transaction_service.take();
    report_to_std_out(&account_service)?;

    return Ok(());
}

fn process_data(transaction_service: &mut TransactionService) -> Result {
    let input_path = args::parse_input_arg()?;
    let mut rdr = reader::build_csv_reader(input_path)?;

    for record in rdr.deserialize() {
        let event: InputEvent = record?;
        let event: TransactionEvent = event.parse()?;

        transaction_service.process_event(event)?;
    }

    return Ok(());
}

fn report_to_std_out(account_service: &AccountService) -> Result {
    let report = account_service.build_report()?;

    let mut wtr = writer::build_csv_writer();

    for account_report in report.iter() {
        wtr.serialize(account_report)?;
    }

    let output = writer::write_to_string(wtr)?;

    println!("{}", output);

    return Ok(());
}
