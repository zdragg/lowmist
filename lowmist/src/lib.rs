pub mod plugins;

use bevy::{app::PluginGroupBuilder, prelude::*};

use crate::plugins::waila::WailaPlugin;
pub use crate::plugins::{
    blocks::BlocksPlugin, camera::MinecraftCameraPlugin, grid::ChunkGridPlugin,
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
