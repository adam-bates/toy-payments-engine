use tpe::Result;

use csv::Writer;

pub fn build_csv_writer() -> Writer<Vec<u8>> {
    return Writer::from_writer(vec![]);
}

pub fn write_to_string(writer: Writer<Vec<u8>>) -> Result<String> {
    let utf8 = writer.into_inner()?;
    let string = String::from_utf8(utf8)?;
    return Ok(string);
}
