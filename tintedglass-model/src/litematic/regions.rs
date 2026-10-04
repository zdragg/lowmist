use std::{collections::HashMap, sync::LazyLock};

use bitvec::prelude::*;
use fastnbt::LongArray;
use glam::{DVec3, IVec3};
use serde::Deserialize;

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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BlockStatePaletteEntry {
    pub name: String,
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

impl BlockStatePaletteEntry {
    pub fn is_air(&self) -> bool {
        self.name == "minecraft:air"
            || self.name == "minecraft:void_air"
            || self.name == "minecraft:cave_air"
    }
}

/// `minecraft:air` is used as fallback.
pub static AIR: LazyLock<BlockStatePaletteEntry> = LazyLock::new(|| BlockStatePaletteEntry {
    name: String::from("minecraft:air"),
    properties: HashMap::new(),
});
/// Reference: <https://github.com/sakura-ryoko/litematica/blob/f7ac844c8134745cd89a6d9690cf3c753fe57465/src/main/java/fi/dy/masa/litematica/schematic/container/LitematicaBitArray.java#L103>
impl Region {
    /// Takes **local** position and returns the corresponding block palette entry.
    /// Returns None if invalid position coordinates are provided.
    pub fn block_at_local(&self, local_pos: IVec3) -> Option<&BlockStatePaletteEntry> {
        let slot = self.flatten_coords(local_pos)?;
        Some(self.get_palette_entry_at(slot))
    }

    /// Takes **global** position and returns the corresponding block palette entry.
    /// Returns None if invalid position coordinates are provided.
    pub fn block_at_global(&self, global_pos: IVec3) -> Option<&BlockStatePaletteEntry> {
        let local_pos = self.global_to_local_pos(global_pos);
        self.block_at_local(local_pos)
    }

    /// Returns an iterator that iterates over every possible local coordinate
    /// and its corresponding palette entry.
    pub fn blocks_local_pos(&self) -> impl Iterator<Item = (IVec3, &BlockStatePaletteEntry)> {
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
    pub fn blocks_global_pos(&self) -> impl Iterator<Item = (IVec3, &BlockStatePaletteEntry)> {
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
    pub fn get_palette_entry_at(&self, slot: usize) -> &BlockStatePaletteEntry {
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
        let bit_slice = words.view_bits::<Lsb0>();

        // Find the bits that the slot points at in the bit slice, then cast to u32
        let palette_index = bit_slice
            .get(slot * bits_per_slot..(slot + 1) * bits_per_slot)
            .map(|bits| bits.load_le::<u32>());
        let Some(palette_index) = palette_index else {
            // Corrupted block state
            return &AIR;
        };

        // Find the corresponding entry:
        self.block_state_palette
            .get(palette_index as usize)
            // Litematica sets invalid palette indices to `minecraft:air`.
            .unwrap_or(&AIR)
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
