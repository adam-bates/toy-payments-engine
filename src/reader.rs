use tpe::Result;

use std::{
    path::PathBuf,
    fs::File,
};

use csv::{Reader, ReaderBuilder, Trim};

pub fn build_csv_reader(filepath: PathBuf) -> Result<Reader<File>> {
    let reader = ReaderBuilder::new()
        .trim(Trim::All)
        .from_path(filepath)?;

    return Ok(reader);
}

