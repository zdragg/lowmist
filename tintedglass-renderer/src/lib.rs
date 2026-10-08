pub mod plugins;

use bevy::{app::PluginGroupBuilder, prelude::*};

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
    }
}
