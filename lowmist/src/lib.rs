pub mod plugins;

use bevy::{app::PluginGroupBuilder, prelude::*};

use crate::plugins::waila::WailaPlugin;
pub use crate::plugins::{
    camera::MinecraftCameraPlugin, grid::ChunkGridPlugin, schematic::SchematicPlugin,
};

pub struct TintedGlassPlugins;

impl PluginGroup for TintedGlassPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(SchematicPlugin)
            .add(MinecraftCameraPlugin)
            .add(ChunkGridPlugin)
            .add(WailaPlugin)
    }
}
