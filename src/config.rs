use tpe::Result;

use log::LevelFilter;
use simple_logger::SimpleLogger;

// Configures application. Namely, logging
// Note: Default log level is WARN, set the RUST_LOG env variable to override this
pub fn configure_app() -> Result {
    SimpleLogger::new()
        .with_level(LevelFilter::Warn)
        .env()
        .init()?;

    Ok(())
}

