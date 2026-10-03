use std::{collections::HashMap, sync::LazyLock};

use bitvec::prelude::*;
use fastnbt::LongArray;
use serde::Deserialize;

/// Reference: <https://github.com/sakura-ryoko/litematica/blob/f7ac844c8134745cd89a6d9690cf3c753fe57465/src/main/java/fi/dy/masa/litematica/schematic/LitematicaSchematic.java#L1761>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Region {
    #[serde(with = "crate::serde::ivec3::xyz")]
    pub position: glam::IVec3,
    /// Size could be negative, because it is calculated by
    /// [coords of block selection 1] - [coords of block selection 2].
    /// If block 1 is to the left of / in front of / below block 2,
    /// then size is going to have a negative value.
    /// Use its absolute value!
    #[serde(with = "crate::serde::ivec3::xyz")]
    pub size: glam::IVec3,

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

/// `minecraft:air` is used as fallback.
pub static AIR: LazyLock<BlockStatePaletteEntry> = LazyLock::new(|| BlockStatePaletteEntry {
    name: String::from("minecraft:air"),
    properties: HashMap::new(),
});
/// Reference: <https://github.com/sakura-ryoko/litematica/blob/f7ac844c8134745cd89a6d9690cf3c753fe57465/src/main/java/fi/dy/masa/litematica/schematic/container/LitematicaBitArray.java#L103>
impl Region {
    /// Flattens the position (relative to the minimum corner of the region)
    /// to a single number that represents the **slot** of the target block within the packed bits.
    ///
    /// Use the result in [`Self::get_palette_entry_at()`] to get the corresponding palette index.
    pub fn flatten_coords_to_slot(&self, region_pos: glam::IVec3) -> Option<usize> {
        let glam::IVec3 {
            x: size_x,
            y: size_y,
            z: size_z,
            ..
        } = self.size.abs(); // Size could be negative
        let glam::IVec3 { x, y, z } = region_pos;

        if x < 0 || x >= size_x || y < 0 || y >= size_y || z < 0 || z >= size_z {
            // Invalid position provided.
            return None;
        }

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
            return &AIR;
        };

        // Find the corresponding entry:
        self.block_state_palette
            .get(palette_index as usize)
            .unwrap_or(&AIR)
    }

    /// Returns an iterator that iterates over every possible coordinate
    /// and its corresponding palette entry.
    pub fn block_state_iter(&self) -> impl Iterator<Item = (glam::IVec3, &BlockStatePaletteEntry)> {
        let glam::IVec3 {
            x: size_x,
            y: size_y,
            z: size_z,
        } = self.size.abs();
        itertools::iproduct!(0..size_y, 0..size_z, 0..size_x).map(|(y, z, x)| {
            let pos = glam::IVec3 { x, y, z };
            let slot = self.flatten_coords_to_slot(pos).unwrap();
            let palette_entry = self.get_palette_entry_at(slot);
            (pos, palette_entry)
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Entity {
    #[serde(rename = "Pos", default)]
    pub pos: glam::DVec3,
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
