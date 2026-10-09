pub mod plugins;

use bevy::{app::PluginGroupBuilder, prelude::*};

use crate::plugins::waila::WailaPlugin;
pub use crate::plugins::{
    camera::MinecraftCameraPlugin, grid::ChunkGridPlugin, schematic::SchematicPlugin,
};

pub struct LowmistPlugins;

impl PluginGroup for LowmistPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(SchematicPlugin)
            .add(MinecraftCameraPlugin)
            .add(ChunkGridPlugin)
            .add(WailaPlugin)
    }
}
