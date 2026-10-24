//! Schematic-related types
//!
//! LoadedSchematic: Holds handle to the schematic YOU WANT rendered.
//! The code checks LoadedSchematic against the actually rendered schematic,
//! and renders your new one if they don't match.
//!
//! Schematic: A wrapper around the actual Litematic struct that can be loaded
//! as a bevy Asset.
//!
//! SchematicLoader: A bevy struct that defines how to load bytes into a SchematicAsset.
//!
//! Usage:
//! Use Bevy's AssetServer to load a .litematic file into a Handle<Schematic>.
//! Then, store that Handle<Schematic> into the LoadedSchematic resource to tell
//! the renderer that you want this schematic rendered.

use std::sync::Arc;

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};
use lowmist_schem::Litematic;

pub(super) struct InnerSchematicPlugin;

impl Plugin for InnerSchematicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadedSchematic>()
            .init_asset::<Schematic>()
            .init_asset_loader::<SchematicLoader>();
    }
}

#[derive(Asset, TypePath)]
pub struct Schematic(Arc<Litematic>);

impl Schematic {
    pub fn get(&self) -> Arc<Litematic> {
        self.0.clone()
    }
}

#[derive(Default, TypePath)]
struct SchematicLoader;

impl AssetLoader for SchematicLoader {
    type Asset = Schematic;
    type Settings = ();
    type Error = BevyError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let schematic = Litematic::parse(&bytes[..])?;
        Ok(Schematic(Arc::new(schematic)))
    }

    fn extensions(&self) -> &[&str] {
        &["litematic"]
    }
}

/// This resource holds a handle pointing at the currently loaded schematic.
///
/// The associated plugin renders the schematic stored within this resource.
///
/// To change the loaded schematic:
/// ```rust
/// use bevy::prelude::*;
/// use bevy::asset::AssetPath;
/// use lowmist::plugins::blocks::schematic::LoadedSchematic;
/// fn load_schematic(
///     mut loaded_handle_resource: ResMut<LoadedSchematic>,
///     asset_server: Res<AssetServer>
/// ) {
///     let schem_path = AssetPath::parse("path_to_schematic");
///     loaded_handle_resource.0 = asset_server.load(schem_path);
/// }
/// ```
///
/// To read the loaded schematic:
/// ```rust
/// use bevy::prelude::*;
/// use lowmist::plugins::blocks::schematic::Schematic;
/// use lowmist::plugins::blocks::schematic::LoadedSchematic;
/// fn read_schematic(
///     loaded_handle_resource: Res<LoadedSchematic>,
///     loaded_schems: Res<Assets<Schematic>>,
/// ) {
///     let loaded_handle = &loaded_handle_resource.0;
///     if let Some(loaded_schem) = loaded_schems.get(loaded_handle) {
///         todo!();
///     }
/// }
/// ```
#[derive(Default, Resource)]
pub struct LoadedSchematic(pub Handle<Schematic>);
