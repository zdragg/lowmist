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
    #[snafu(
        visibility(pub(crate)),
        display("More than u16::MAX global palette entries created")
    )]
    PaletteOverflow { source: std::num::TryFromIntError },
    #[snafu(display("Could not read file bytes"))]
    FileRead { source: std::io::Error },
}

/// Parses any [`Read`] that represents a .litematic file and returns [`Litematic`].
pub fn parse_litematic(schematic: impl Read) -> Result<Litematic, Error> {
    let mut decoder = GzDecoder::new(schematic);

    let mut data = vec![];
    decoder.read_to_end(&mut data).context(FileReadSnafu)?;

    let schematic_result: Result<Litematic, _> = from_bytes(data.as_slice());
    let mut schematic = schematic_result.context(ParseSchematicSnafu)?;

    schematic.build_global_block_palette()?;

    Ok(schematic)
}
