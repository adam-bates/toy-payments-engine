use tpe::Result;

use simple_logger::SimpleLogger;

pub fn configure_app() -> Result {
    SimpleLogger::new().init()?;
    return Ok(());
}

