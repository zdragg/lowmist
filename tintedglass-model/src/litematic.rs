mod regions;
pub use regions::*;
mod metadata;
pub use metadata::*;

use std::collections::HashMap;

use glam::IVec3;
use serde::{Deserialize, Deserializer};

/// Reference: <https://github.com/sakura-ryoko/litematica/blob/f7ac844c8134745cd89a6d9690cf3c753fe57465/src/main/java/fi/dy/masa/litematica/schematic/LitematicaSchematic.java#L1702>
/// This parser does not support version 1.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Litematic {
    #[serde(deserialize_with = "version_blocker")]
    pub version: i32,
    #[serde(default)]
    pub sub_version: i32,
    /// [https://minecraft.wiki/w/Data_version](https://minecraft.wiki/w/Data_version)
    pub minecraft_data_version: i32,
    pub metadata: Metadata,
    pub regions: HashMap<String, Region>,
}

/// Serde deserializer that only allows versions 2 to 7.
fn version_blocker<'de, D: Deserializer<'de>>(de: D) -> Result<i32, D::Error> {
    let v = i32::deserialize(de)?;
    match v {
        2..=7 => Ok(v),
        _ => Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Signed(v.into()),
            &"schematic version 2 to 7",
        )),
    }
}

impl Litematic {
    /// Iterates through regions until it finds a block (palette) for
    /// the specified position.
    /// Returns None if none of the regions has a block at the position.
    pub fn block_at(&self, pos: IVec3) -> Option<&BlockStatePaletteEntry> {
        self.regions
            .iter()
            .find_map(|(_name, region)| region.block_at_global(pos))
    }

    /// Returns an iterator that iterates over every valid coordinate with a corresponding
    /// palette entry. Each valid coordinate only appears once (hence deduplicated)
    pub fn blocks_dedup(&self) -> impl Iterator<Item = (IVec3, &BlockStatePaletteEntry)> {
        let regions = self.regions.values();
        // Creates a vector where prev_intersect[i] contains a list of references to
        // all "previous" Regions that intersect (overlap) with the i-th region in the regions.values() list.
        //
        // Example: If region 3 overlaps with region 0, 2, 4, but not region 1,
        // then prev_intersects[3] contains vec![&region0, &region2].
        // Region 1 is excluded because it does not overlap with region 3;
        // region 4 is excluded because the blocks in region 4 wouldn't have been iterated over yet.
        // When we reach region 4 in this example, it will contain region 3 as one of the regions
        // that intersected itself.
        //
        // Example: prev_intersects[0] is always an empty vector.
        let prev_intersects: Vec<Vec<_>> = regions
            .clone()
            .enumerate()
            .map(|(i, current)| {
                // Take all regions before this one that intersects
                let prev_intersects: Vec<_> = self
                    .regions
                    .iter()
                    .map(|(_, prev)| prev)
                    .take(i)
                    .filter(|prev| current.intersects(prev))
                    .collect();
                prev_intersects
            })
            .collect();

        // For each Region (re-iterated), go through the corresponding list of references constructed earlier,
        // and create a flattened iterator over all deduplicated blocks NOT within the bounding boxes of these
        // previously iterated regions.
        regions
            .zip(prev_intersects)
            .flat_map(|(region, prev_intersects)| {
                region.blocks_global_pos().filter(move |(pos, _)| {
                    !prev_intersects
                        .iter()
                        .any(|prev| prev.contains_global(*pos))
                })
            })
    }
}
