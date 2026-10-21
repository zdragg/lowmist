//! Minecraft Resource Pack handler.
//!
//! - downloads the default resource pack from the internet
//! - parses the files using `lowmist-respack`
//! - loads the parsed files into the asset server
//! - generates a new ModelBaker dynamically when loaded resource packs change

use std::{collections::BTreeMap, str::FromStr};

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};
use lowmist_respack::{
    ResourceLocation,
    blockstate::{BlockstateDispatcher, OneOrManyVariants},
    model::Model,
    texture::TextureMcmeta,
};

use crate::plugins::blocks::schematic::{LoadedSchematic, Schematic};

const DEFAULT_RESOURCE_PACK_URL: &str = "https://cdn.jsdelivr.net/gh/zdragg/minecraft-java-default-resource-pack@26.3/DefaultResourcePack.zip";

pub(super) struct InnerMinecraftAssetPlugin;

impl Plugin for InnerMinecraftAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ResourcePack>()
            .init_asset_loader::<ResourcePackLoader>()
            .add_systems(Startup, load_default_resource_pack)
            .add_systems(
                Update,
                build_model_baker.run_if(
                    resource_changed::<ResourcePackStack>
                        .or_eager(resource_changed::<LoadedSchematic>),
                ),
            );
    }
}

#[derive(Resource)]
struct ResourcePackStack(Vec<Handle<ResourcePack>>);

impl ResourcePackStack {
    fn new(base: Handle<ResourcePack>) -> Self {
        Self(vec![base])
    }
}

fn load_default_resource_pack(mut commands: Commands, asset_server: Res<AssetServer>) {
    let resource_pack = asset_server.load(DEFAULT_RESOURCE_PACK_URL);
    commands.insert_resource(ResourcePackStack::new(resource_pack));
}

#[derive(Asset, TypePath)]
pub struct ResourcePack(lowmist_respack::ResourcePack);

impl ResourcePack {
    pub fn inner(&self) -> &lowmist_respack::ResourcePack {
        &self.0
    }
}

#[derive(Default, TypePath)]
struct ResourcePackLoader;

impl AssetLoader for ResourcePackLoader {
    type Asset = ResourcePack;
    type Settings = ();
    type Error = BevyError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;

        let pack = lowmist_respack::ResourcePack::load(&buf)?;

        Ok(ResourcePack(pack))
    }

    fn extensions(&self) -> &[&str] {
        &["zip"]
    }
}

/// A **REVERSED** stack of references to ResourcePack.
/// `self.0.last()` returns Some(vanilla resource pack).
struct PackStackRef<'a>(Vec<&'a ResourcePack>);
impl<'a> PackStackRef<'a> {
    fn from_handles(
        stack: &ResourcePackStack,
        assets: &'a Assets<ResourcePack>,
    ) -> Result<Self, BevyError> {
        stack
            .0
            .iter()
            .rev() // REVERSAL HAPPENS HERE
            .map(|handle| {
                assets
                    .get(handle)
                    .context("resource pack on the stack not found in asset server")
            })
            .collect::<Result<_>>()
            .map(Self)
    }
    // blockstate from each pack is MERGED, so every single one is returned
    fn get_blockstate(
        &'a self,
        id: &ResourceLocation,
    ) -> impl Iterator<Item = &'a BlockstateDispatcher> {
        self.0.iter().filter_map(|pack| pack.0.blockstates.get(id))
    }
    // only topmost model is used, so only one returned
    fn get_model(&'a self, id: &ResourceLocation) -> Option<&'a Model> {
        self.0.iter().find_map(|pack| pack.0.models.get(id))
    }
    // only topmost texture is used, so only one returned
    fn get_texture(
        &'a self,
        id: &ResourceLocation,
    ) -> Option<(&'a [u8], Option<&'a TextureMcmeta>)> {
        self.0.iter().find_map(|pack| {
            pack.0
                .textures
                .get(id)
                .map(|texture| (texture.as_slice(), pack.0.textures_mcmeta.get(id)))
        })
    }

    fn get_quads(
        &self,
        block_name: &str,
        block_properties: &BTreeMap<String, String>,
    ) -> Option<()> {
        let id = ResourceLocation::from_str(block_name).into_ok();
        let model_variants = self.get_blockstate(&id).find_map(|blockstate_dispatcher| {
            // Does this blockstate dispatcher have definitions for `block_properties`?
            blockstate_dispatcher.dispatch(block_properties)
        })?;
        Some(())
    }
}

#[derive(Resource, PartialEq)]
struct ModelBaker;

fn build_model_baker(
    mut commands: Commands,
    resource_pack_assets: Res<Assets<ResourcePack>>,
    resource_pack_stack: Res<ResourcePackStack>,
    schem_assets: Res<Assets<Schematic>>,
    loaded_schem: Res<LoadedSchematic>,
) -> Result<()> {
    let Some(schem) = schem_assets.get(&loaded_schem.0) else {
        return Ok(()); // No schematic loaded
    };

    let pack_stack = PackStackRef::from_handles(&resource_pack_stack, &resource_pack_assets)?;

    // Fetch the blockstates used by the schematic
    for (_, bs) in schem.inner().palettes() {
        let block_name = &bs.name; // "minecraft:grass_block"
        let block_properties = &bs.properties; // "snowy=false"

        let quads = pack_stack.get_quads(block_name, block_properties);
    }

    commands.insert_resource_if_neq(ModelBaker);
    Ok(())
}
