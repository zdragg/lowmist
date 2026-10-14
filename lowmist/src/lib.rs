#![forbid(unsafe_code)]
#![allow(clippy::too_many_arguments)]

pub mod plugins;

use bevy::{app::PluginGroupBuilder, prelude::*};

pub use crate::plugins::{
    blocks::BlocksPlugin, camera::MinecraftCameraPlugin, grid::ChunkGridPlugin, waila::WailaPlugin,
};

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
