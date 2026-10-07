mod error;
pub mod plugins;

pub use error::{Error, Result};

use bevy::{app::PluginGroupBuilder, prelude::*};

pub use crate::plugins::schematic::Schematic;
pub use crate::plugins::{
    camera::MinecraftCameraPlugin, grid::GridPlugin, schematic::SchematicPlugin,
};

pub struct TintedGlassPlugins;

impl PluginGroup for TintedGlassPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(SchematicPlugin)
            .add(MinecraftCameraPlugin)
            .add(GridPlugin)
    }
}
