use tpe::Result;

use log::LevelFilter;
use simple_logger::SimpleLogger;

pub fn configure_app() -> Result {
    SimpleLogger::new()
        .with_level(LevelFilter::Warn)
        .env()
        .init()?;

    return Ok(());
}

