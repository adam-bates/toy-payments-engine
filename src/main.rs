mod args;
mod config;
mod reader;
mod writer;

use tpe::{
    input::InputEvent,
    events::TransactionEvent,
    Result, services::{TransactionService, AccountService, TransactionServiceError, AccountServiceError},
};

fn main() -> Result {
    config::configure_app()?;

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

        let res = transaction_service.process_event(event);

        if let Err(e) = res {
            match e.downcast_ref::<TransactionServiceError>() {
                Some(e) => {
                    log::warn!("{e:?}");
                    continue;
                },
                _ => {},
            }

            match e.downcast_ref::<AccountServiceError>() {
                Some(e) => match e {
                    AccountServiceError::InvalidWithdrawal(msg) => {
                        log::warn!("{:?}", AccountServiceError::InvalidWithdrawal(msg.to_string()));
                        continue;
                    },
                    _ => {},
                },
                _ => {},
            }

            Err(e)?
        }
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
