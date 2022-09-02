use tpe::Result;

use std::{
    env,
    fs,
    path::PathBuf,
};

use anyhow::Context;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum InputArgsError {
    #[error("Couldn't parsing input arguments: {0}")]
    Parse(String),

    #[error("File not found: {0}")]
    FileNotFound(String),
}

pub fn parse_input_arg() -> Result<PathBuf> {
    let filename = env::args().skip(1).next()
        .ok_or_else(|| InputArgsError::Parse(format!("First argument must be the input file.")))?;

    let path = fs::canonicalize(filename.clone())
        .with_context(|| InputArgsError::FileNotFound(filename))?;

    return Ok(path);
}
