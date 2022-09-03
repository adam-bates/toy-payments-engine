use tpe::Result;

use std::{fs::File, path::PathBuf};

use csv::{Reader, ReaderBuilder, Trim};

/// Builds an empty csv reader
pub fn build_csv_reader(filepath: PathBuf) -> Result<Reader<File>> {
    let reader = ReaderBuilder::new().trim(Trim::All).from_path(filepath)?;

    Ok(reader)
}
