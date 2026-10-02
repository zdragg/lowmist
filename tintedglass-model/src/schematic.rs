mod regions;
pub use regions::*;
mod metadata;
pub use metadata::*;

use std::collections::HashMap;

use serde::{Deserialize, Deserializer};

/// Reference: <https://github.com/sakura-ryoko/litematica/blob/f7ac844c8134745cd89a6d9690cf3c753fe57465/src/main/java/fi/dy/masa/litematica/schematic/LitematicaSchematic.java#L1702>
/// This parser ONLY supports version 7 for now.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Schematic {
    #[serde(deserialize_with = "version_7")]
    pub version: i32,
    #[serde(default)]
    pub sub_version: i32,
    /// [https://minecraft.wiki/w/Data_version](https://minecraft.wiki/w/Data_version)
    pub minecraft_data_version: i32,
    pub metadata: Metadata,
    pub regions: HashMap<String, Region>,
}

/// Serde deserializer that blocks all non-v7 schematic files.
fn version_7<'de, D: Deserializer<'de>>(de: D) -> Result<i32, D::Error> {
    let v = i32::deserialize(de)?;
    match v {
        7 => Ok(v),
        _ => Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Signed(v.into()),
            &"schematic version 7",
        )),
    }
}
