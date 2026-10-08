mod regions;
use fastnbt::from_bytes;
use flate2::read::GzDecoder;
use glam::IVec3;
use indexmap::IndexSet;
pub use regions::*;
mod metadata;
pub use metadata::*;
use snafu::ResultExt;

use std::{collections::BTreeMap, io::Read};

use serde::{Deserialize, Deserializer};

/// Newtype that represents a block palette ID, used in the schematic-level
/// deduplicated block palette.
#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Clone, Copy)]
pub struct BlockId(u16);

impl BlockId {
    pub fn id(&self) -> u16 {
        self.0
    }
    pub const AIR: BlockId = BlockId(0);
    pub fn is_air(&self) -> bool {
        self.0 < 3 // palette 0, 1 and 2 is "minecraft:{air, cave_air, void_air}"
    }
}

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
    pub regions: BTreeMap<String, Region>,

    /// A schematic-level deduplicated block palette used for a unified access
    /// to all block palettes under each region.
    ///
    /// Note: not part of the Litematica schema.
    #[serde(skip)]
    pub(crate) global_block_state_palette: IndexSet<BlockStatePaletteEntry>,
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

impl Litematic {
    /// Parses any [`Read`] that represents a .litematic file and returns [`Litematic`].
    pub fn parse(bytes: impl Read) -> Result<Self, Error> {
        let mut decoder = GzDecoder::new(bytes);

        let mut data = vec![];
        decoder.read_to_end(&mut data).context(FileReadSnafu)?;

        let schematic_result: Result<Litematic, _> = from_bytes(data.as_slice());
        let mut schematic = schematic_result.context(ParseSchematicSnafu)?;

        schematic.build_global_block_palette()?;

        Ok(schematic)
    }
    /// Iterates through regions until it finds a block id for
    /// the specified position.
    /// Returns None if none of the regions has a block at the position.
    pub fn block_at(&self, pos: IVec3) -> Option<BlockId> {
        self.regions
            .iter()
            .find_map(|(_name, region)| region.block_at_global(pos))
    }

    /// Returns an iterator that iterates over every valid coordinate with a corresponding
    /// palette id. Each valid coordinate only appears once (hence deduplicated)
    pub fn blocks(&self) -> impl Iterator<Item = (IVec3, BlockId)> {
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
                let prev_intersects: Vec<_> = regions
                    .clone()
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

    pub fn blocks_without_air(&self) -> impl Iterator<Item = (IVec3, BlockId)> {
        self.blocks().filter(|(_pos, block)| !block.is_air())
    }

    /// Finds the minimum corner of the bounding box encompassing all regions.
    pub fn min_corner(&self) -> IVec3 {
        self.regions
            .values()
            .fold(IVec3::MAX, |acc, r| acc.min(r.min_corner()))
    }

    /// Finds the maximum corner of the bounding box encompassing all regions.
    pub fn max_corner(&self) -> IVec3 {
        self.regions
            .values()
            .fold(IVec3::MIN, |acc, r| acc.max(r.max_corner()))
    }

    /// Returns (min_corner, max_corner)
    pub fn bounds(&self) -> (IVec3, IVec3) {
        (self.min_corner(), self.max_corner())
    }

    /// Returns the Palette Entry corresponding to the BlockId.
    pub fn palette_entry(&self, id: BlockId) -> &BlockStatePaletteEntry {
        self.global_block_state_palette
            .get_index(id.0 as usize)
            .expect("every BlockId should always have a corresponding BlockStatePaletteEntry")
    }

    /// Builds the globally used, deduplicated block palette.
    pub(crate) fn build_global_block_palette(&mut self) -> Result<(), crate::Error> {
        if !self.global_block_state_palette.is_empty() {
            panic!("global palette already built")
        }
        self.global_block_state_palette
            .insert(BlockStatePaletteEntry {
                name: "minecraft:air".into(),
                properties: BTreeMap::new(),
            }); // BlockId(0) for the fallback entry

        self.global_block_state_palette
            .insert(BlockStatePaletteEntry {
                name: "minecraft:cave_air".into(),
                properties: BTreeMap::new(),
            });

        self.global_block_state_palette
            .insert(BlockStatePaletteEntry {
                name: "minecraft:void_air".into(),
                properties: BTreeMap::new(),
            });

        for region in self.regions.values_mut() {
            for entry in &region.block_state_palette {
                let (global_index, _exists) =
                    self.global_block_state_palette.insert_full(entry.clone());
                let id = BlockId(u16::try_from(global_index).context(PaletteOverflowSnafu)?);
                region.local_to_global.push(id);
            }
        }
        Ok(())
    }

    pub fn palettes(&self) -> impl Iterator<Item = (BlockId, &BlockStatePaletteEntry)> {
        self.global_block_state_palette
            .iter()
            .enumerate()
            .map(|(id, entry)| (BlockId(id as u16), entry))
    }
}
