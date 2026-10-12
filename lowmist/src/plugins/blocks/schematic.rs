//! Schematic-related types
//!
//! LoadedSchematic: Holds handle to the schematic YOU WANT rendered.
//! The code checks LoadedSchematic against the actually rendered schematic,
//! and renders your new one if they don't match.
//!
//! SchematicAsset: A wrapper around the actual Schematic struct that can be loaded
//! as a bevy Asset.
//!
//! SchematicAssetLoader: A bevy struct that defines how to load bytes into a SchematicAsset.
//!
//! Usage:
//! Use Bevy's AssetServer to load a .litematic file into a Handle<SchematicAsset>.
//! Then, store that Handle<SchematicAsset> into the LoadedSchematic resource to tell
//! the renderer that you want this schematic rendered.

use std::sync::Arc;

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};
use lowmist_schematic::Litematic;

pub(super) struct InnerSchematicPlugin;

impl Plugin for InnerSchematicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadedSchematic>()
            .init_asset::<SchematicAsset>()
            .init_asset_loader::<SchematicAssetLoader>();
    }
}

#[derive(Asset, TypePath)]
pub struct SchematicAsset(Arc<Litematic>);

impl SchematicAsset {
    pub fn inner(&self) -> Arc<Litematic> {
        self.0.clone()
    }
}

#[derive(Default, TypePath)]
struct SchematicAssetLoader;

impl AssetLoader for SchematicAssetLoader {
    type Asset = SchematicAsset;
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
        Ok(SchematicAsset(Arc::new(schematic)))
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
/// fn load_schematic(
///     mut loaded_handle_resource: ResMut<LoadedSchematic>,
///     asset_server: Res<AssetServer>
/// ) {
///     let schem_asset_path = todo!();
///     loaded_handle_resource.0 = asset_server.load(schem_asset_path);
/// }
/// ```
///
/// To read the loaded schematic:
/// ```rust
/// fn read_schematic(
///     loaded_handle_resource: Res<LoadedSchematic>,
///     loaded_assets: Res<Assets<SchematicAsset>>,
/// ) {
///     let loaded_handle = &loaded_handle_resource.0;
///     if let Some(loaded_asset) = loaded_assets.get(loaded_handle) {
///         todo!();
///     }
/// }
/// ```
#[derive(Default, Resource)]
pub struct LoadedSchematic(pub Handle<SchematicAsset>);
