#![forbid(unsafe_code)]
#![allow(clippy::too_many_arguments)]

pub mod plugins;

use bevy::{app::PluginGroupBuilder, prelude::*};

pub use crate::plugins::{
    blocks::BlocksPlugin, camera::MinecraftCameraPlugin, grid::ChunkGridPlugin, waila::WailaPlugin,
};

#[cfg(target_family = "wasm")]
pub use wasm_bindgen_rayon::init_thread_pool;

pub struct LowmistPlugins;

impl PluginGroup for LowmistPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(BlocksPlugin)
            .add(MinecraftCameraPlugin)
            .add(ChunkGridPlugin)
            .add(WailaPlugin)
    }
}
