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

/// Pases the input arguments, requiring the first and only argument to be a valid filepath
pub fn parse_input_arg() -> Result<PathBuf> {
    let filename = env::args().nth(1)
        .ok_or_else(|| InputArgsError::Parse("First argument must be the input file.".to_string()))?;

    let path = fs::canonicalize(filename.clone())
        .with_context(|| InputArgsError::FileNotFound(filename))?;

    Ok(path)
}
