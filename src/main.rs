mod args;
mod config;
mod reader;
mod writer;

use tpe::{
    events::TransactionEvent,
    input::InputEvent,
    services::{AccountService, AccountServiceError, TransactionService, TransactionServiceError},
    Result,
};

fn main() -> Result {
    config::configure_app()?;

    log::debug!("Application configured. Beginning process...");

    let mut transaction_service = tpe::build_transaction_service();
    process_data(&mut transaction_service)?;

    log::debug!("Process complete. Beginning report...");

    let account_service = transaction_service.take();
    report_to_std_out(&account_service)?;

    log::debug!("Application finished successfully!");

    return Ok(());
}

fn process_data(transaction_service: &mut TransactionService) -> Result {
    let input_path = args::parse_input_arg()?;
    log::debug!("Found filepath as input arg: {input_path:?}");

    let mut rdr = reader::build_csv_reader(input_path)?;

    log::debug!("Deserializing reader...");
    for record in rdr.deserialize() {
        log::debug!("Prasing record into InputEvent: {record:?}");
        let event: InputEvent = record?;

        log::debug!("Parsing input event into TransactionEvent: {event:?}");
        let event: TransactionEvent = event.parse()?;

        log::debug!("Processessing transaction event: {event:?}");
        let res = transaction_service.process_event(event);

        if let Err(e) = res {
            match e.downcast_ref::<TransactionServiceError>() {
                Some(e) => {
                    log::warn!("{e}");
                    continue;
                }
                _ => {}
            }

            match e.downcast_ref::<AccountServiceError>() {
                Some(e) => {
                    log::warn!("{e}");
                    continue;
                },
                _ => {}
            }

            log::error!("Unrecoverable: {e}");

            Err(e)?
        }
    }

    return Ok(());
}

fn report_to_std_out(account_service: &AccountService) -> Result {
    let report = account_service.build_report()?;
    log::debug!("Successfully built reports for {} accounts", report.len());

    let mut wtr = writer::build_csv_writer();

    log::debug!("Serializing reports...");
    for account_report in report.iter() {
        log::debug!("Serializing report: {account_report:?}");
        wtr.serialize(account_report)?;
    }

    let output = writer::write_to_string(wtr)?;

    log::debug!("Writing to stdout: {output:?}");
    println!("{}", output);

    return Ok(());
}
