use std::collections::{BTreeMap, HashMap};

use fastnbt::LongArray;
use glam::{DVec3, IVec3};
use serde::Deserialize;

use crate::BlockId;

/// Reference: <https://github.com/sakura-ryoko/litematica/blob/f7ac844c8134745cd89a6d9690cf3c753fe57465/src/main/java/fi/dy/masa/litematica/schematic/LitematicaSchematic.java#L1761>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Region {
    #[serde(with = "crate::serde::ivec3::xyz")]
    pub position: IVec3,
    /// Size could be negative, because it is calculated by
    /// [coords of block selection 1] - [coords of block selection 2].
    /// If block 1 is to the left of / in front of / below block 2,
    /// then size is going to have a negative value.
    /// Use its absolute value!
    #[serde(with = "crate::serde::ivec3::xyz")]
    pub size: IVec3,

    pub block_states: LongArray,
    pub block_state_palette: Vec<BlockStatePaletteEntry>,

    pub entities: Vec<Entity>,
    pub tile_entities: Vec<TileEntity>,

    // Added in Litematica v3
    #[serde(default)]
    pub pending_block_ticks: Vec<PendingBlockTick>,
    // Added in Litematica v5
    #[serde(default)]
    pub pending_fluid_ticks: Vec<PendingFluidTick>,

    /// A map of which global BlockId
    /// each palette entry in `block_state_palette` corresponds to.
    ///
    /// Note: not part of the Litematica schema.
    #[serde(skip)]
    pub(crate) local_to_global: Vec<BlockId>,
}

#[derive(Debug, Deserialize, Hash, PartialEq, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BlockStatePaletteEntry {
    pub name: String,
    #[serde(default)]
    pub properties: BTreeMap<String, String>,
}

impl BlockStatePaletteEntry {
    pub fn is_opaque(&self) -> bool {
        true // TODO: check if full, opaque block or not
    }
}

/// Reference: <https://github.com/sakura-ryoko/litematica/blob/f7ac844c8134745cd89a6d9690cf3c753fe57465/src/main/java/fi/dy/masa/litematica/schematic/container/LitematicaBitArray.java#L103>
impl Region {
    /// Takes **local** position and returns the corresponding block palette id.
    /// Returns None if invalid position coordinates are provided.
    pub fn block_at_local(&self, local_pos: IVec3) -> Option<BlockId> {
        let slot = self.flatten_coords(local_pos)?;
        Some(self.get_palette_id_at(slot))
    }

    /// Takes **global** position and returns the corresponding block palette id.
    /// Returns None if invalid position coordinates are provided.
    pub fn block_at_global(&self, global_pos: IVec3) -> Option<BlockId> {
        let local_pos = self.global_to_local_pos(global_pos);
        self.block_at_local(local_pos)
    }

    /// Returns an iterator that iterates over every possible local coordinate
    /// and its corresponding palette id.
    pub fn blocks_local_pos(&self) -> impl Iterator<Item = (IVec3, BlockId)> {
        let IVec3 {
            x: size_x,
            y: size_y,
            z: size_z,
        } = self.size.abs();
        itertools::iproduct!(0..size_y, 0..size_z, 0..size_x).map(|(y, z, x)| {
            let local_pos = IVec3 { x, y, z };
            let block = self.block_at_local(local_pos).unwrap();
            (local_pos, block)
        })
    }

    /// Wraps the local_pos equivalent function, but uses global coordinate.
    pub fn blocks_global_pos(&self) -> impl Iterator<Item = (IVec3, BlockId)> {
        self.blocks_local_pos()
            .map(|(local_pos, block)| (self.local_to_global_pos(local_pos), block))
    }

    /// Returns the global coordinates that represents the local (0, 0, 0).
    ///
    /// Useful for translating between local and global coordinates.
    pub fn min_corner(&self) -> IVec3 {
        // Consider only one of the axes: x = 100.
        // Now there may be two scenarios for size_x:
        // 1. Size is positive. Let size_x = 5.
        // Then we have 5 blocks enclosed by the region:
        // 100, 101, 102, 103, 104.
        // The minimum corner of the region is therefore 100.
        // To calculate this, we start from x=100 and add an offset of zero.
        //
        // 2. Size is negative. Let size_x = -5.
        // Then we have 5 blocks as well:
        // 96, 97, 98, 99, 100.
        // The minimum corner of the region is now 96.
        // We calculate this by starting from x=100, and adding an offset of -4, or (-5 + 1).
        //
        // An easy way to calculate the offset is (size_x + 1).min(0).
        //              OFFSET HERE ->
        self.position + (self.size + IVec3::ONE).min(IVec3::ZERO)
    }

    /// Returns the global coordinates that represents the corner
    /// opposite to the minimum corner in the rectangular prism that is
    /// the region.
    pub fn max_corner(&self) -> IVec3 {
        self.min_corner() + self.size.abs() - IVec3::ONE
    }

    /// Returns true if the provided local position has a corresponding block
    /// in this region. 0..abs(size)
    pub fn contains(&self, local_pos: IVec3) -> bool {
        local_pos.cmplt(self.size.abs()).all() && local_pos.cmpge(IVec3::ZERO).all()
    }

    /// Returns true if the provided global position has a corresponding block
    /// in this region.
    pub fn contains_global(&self, global_pos: IVec3) -> bool {
        self.contains(self.global_to_local_pos(global_pos))
    }

    /// Returns true if the provided other region intersects (overlaps) with this region.
    pub fn intersects(&self, other: &Self) -> bool {
        self.min_corner().cmple(other.max_corner()).all()
            && self.max_corner().cmpge(other.min_corner()).all()
    }

    /// Translates coordinates from global to local position, relative to the minimum corner.
    ///
    /// Note: The function would work even if the local position doesn't
    /// point at a valid block within the region.
    pub fn global_to_local_pos(&self, global_pos: IVec3) -> IVec3 {
        global_pos - self.min_corner()
    }

    /// Translates coordinates from local to global position.
    pub fn local_to_global_pos(&self, local_pos: IVec3) -> IVec3 {
        local_pos + self.min_corner()
    }

    /// Translates coordinates from local to global position.

    /// Takes the local position and flattens it to a single number
    /// that represents the **slot** of the target block within the packed bits.
    ///
    /// Use the result in [`Self::get_palette_entry_at()`] to get the corresponding palette index.
    pub fn flatten_coords(&self, local_pos: IVec3) -> Option<usize> {
        if !self.contains(local_pos) {
            return None;
        }

        let IVec3 {
            x: size_x,
            z: size_z,
            ..
        } = self.size.abs();
        let IVec3 { x, y, z } = local_pos;

        Some((y * size_x * size_z + z * size_x + x) as usize)
    }

    /// Find the palette index that provided slot in the packed bits represents,
    /// then return the palette entry that the palette index points at.
    /// Invalid slots and invalid palette index returns `minecraft:air`.
    pub fn get_palette_id_at(&self, slot: usize) -> BlockId {
        // Bits used per palette entry is equal to the shortest bit length required
        // to store the index of the final palette entry.
        // Exception: each entry needs at least 2 bits.
        // e.g. 20 entries total -> 5 bits per entry (19 -> 10011, needs 5 bits).
        // e.g. 1 entry total -> 2 bits per entry anyway.
        let bits_per_slot =
            2.max(i32::BITS - (self.block_state_palette.len() as i32 - 1).leading_zeros()) as usize;

        // Turn the block states, deref coerced into &[i64] then cast into &[u64],
        // into a contiguous bit slice.
        let words: &[u64] = bytemuck::cast_slice(&self.block_states);

        let Some(number) = Self::get_within_packed_bits(slot, bits_per_slot, words) else {
            // Corrupted blockstate
            return BlockId::AIR;
        };

        // Find the corresponding id:
        self.local_to_global
            .get(number as usize)
            .copied()
            .unwrap_or(BlockId::AIR)
    }

    fn get_within_packed_bits(slot: usize, bits_per_slot: usize, bytes: &[u64]) -> Option<u64> {
        let start = slot * bits_per_slot; // The starting bit of the number, inclusive
        let word_index = start / 64; // words[word_index]
        let offset = start % 64; // Which bit within words[word_index]

        let fits_in_one_word = offset + bits_per_slot <= 64;

        let mask: u64 = (0b1 << bits_per_slot) - 1; // e.g. if 5 bits per slot, then mask is 0b0000...00011111

        if fits_in_one_word {
            Some((bytes.get(word_index)? >> offset) & mask)
        } else {
            let lo = bytes.get(word_index)? >> offset;
            let hi = bytes.get(word_index + 1)? << (64 - offset);
            Some((lo | hi) & mask)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Entity {
    #[serde(rename = "Pos", default)]
    pub pos: DVec3,
    #[serde(flatten)]
    pub data: HashMap<String, fastnbt::Value>,
}

#[derive(Debug, Deserialize)]
pub struct TileEntity {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    #[serde(flatten)]
    pub data: HashMap<String, fastnbt::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PendingBlockTick {
    pub block: String,
    pub time: i32,
    pub priority: i32,
    pub sub_tick: i64,
    #[serde(rename = "x")]
    pub x: i32,
    #[serde(rename = "y")]
    pub y: i32,
    #[serde(rename = "z")]
    pub z: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PendingFluidTick {
    pub fluid: String,
    pub time: i32,
    pub priority: i32,
    pub sub_tick: i64,
    #[serde(rename = "x")]
    pub x: i32,
    #[serde(rename = "y")]
    pub y: i32,
    #[serde(rename = "z")]
    pub z: i32,
}
