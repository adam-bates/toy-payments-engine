mod args;
mod config;
mod reader;
mod writer;

use tpe::{input::InputEvent, AccountSnapshots, Ledger, Result};

fn main() -> Result {
    config::configure_app()?;

    log::debug!("Application configured. Beginning process...");

    let mut ledger = Ledger::new();
    let mut snapshots = AccountSnapshots::new();

    process_data(&mut ledger, &mut snapshots)?;

    log::debug!("Process complete. Beginning report...");

    report_to_std_out(&snapshots)?;

    log::debug!("Application finished successfully!");

    Ok(())
}

/// Read input file, process, and store results
fn process_data(ledger: &mut Ledger, snapshots: &mut AccountSnapshots) -> Result {
    let input_path = args::parse_input_arg()?;
    log::debug!("Found filepath as input arg: {input_path:?}");

    let mut rdr = reader::build_csv_reader(input_path)?;

    log::debug!("Deserializing reader...");
    for record in rdr.deserialize::<InputEvent>() {
        log::debug!("Parsing record into InputEvent: {record:?}");
        let input_event = match record {
            Ok(input_event) => input_event,
            Err(e) => {
                log::warn!("{e}");
                continue;
            }
        };

        log::debug!("Parsing input_event into Transaction: {input_event:?}");
        let tx = match input_event.parse_transaction() {
            Ok(tx) => tx,
            Err(e) => {
                log::warn!("{e}");
                continue;
            }
        };

        let client_id = tx.client_id;

        log::debug!("Appending transaction to ledger: {tx:?}");
        let ledger_idx = ledger.append(tx);

        log::debug!("Appended at index: {ledger_idx}");

        let snapshot = snapshots.find_mut_or_create(client_id);

        log::debug!("Applying tx {ledger_idx} to snapshot: {snapshot:?}");
        if let Err(e) = snapshot.apply_transactions(ledger) {
            log::warn!("{e}");
        }
    }

    Ok(())
}

/// Build report from results, and write report to stdout
fn report_to_std_out(snapshots: &AccountSnapshots) -> Result {
    let report = snapshots.build_report()?;
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

    Ok(())
}
