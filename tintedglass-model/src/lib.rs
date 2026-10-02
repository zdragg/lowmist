mod serde;

mod schematic;
pub use schematic::*;

use std::io::Read;

use fastnbt::from_bytes;
use flate2::read::GzDecoder;
use snafu::ResultExt;

#[derive(Debug, snafu::Snafu)]
#[snafu(display("Could not parse bytes as a schematic"))]
pub struct ParseSchematicError {
    source: fastnbt::error::Error,
}

pub fn parse_schematic(schematic: impl Read) -> Result<Schematic, ParseSchematicError> {
    let mut decoder = GzDecoder::new(schematic);

    let mut data = vec![];
    decoder.read_to_end(&mut data).unwrap();

    let schematic_result: Result<Schematic, _> = from_bytes(data.as_slice());
    let schematic = schematic_result.context(ParseSchematicSnafu)?;
    Ok(schematic)
}
