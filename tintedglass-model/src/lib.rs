mod serde;

mod litematic;
pub use litematic::*;

use std::io::Read;

use fastnbt::from_bytes;
use flate2::read::GzDecoder;
use snafu::ResultExt;

#[derive(Debug, snafu::Snafu)]
pub enum Error {
    #[snafu(display("Could not parse bytes as a schematic"))]
    ParseSchematic { source: fastnbt::error::Error },
    #[snafu(display("Could not read file bytes"))]
    FileRead { source: std::io::Error },
}

pub fn parse_litematic(schematic: impl Read) -> Result<Litematic, Error> {
    let mut decoder = GzDecoder::new(schematic);

    let mut data = vec![];
    decoder.read_to_end(&mut data).context(FileReadSnafu)?;

    let schematic_result: Result<Litematic, _> = from_bytes(data.as_slice());
    let schematic = schematic_result.context(ParseSchematicSnafu)?;
    Ok(schematic)
}
