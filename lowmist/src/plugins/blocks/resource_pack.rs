//! A few types that handle the Minecraft Resource Pack.
//!
//! This file handles:
//! - downloading the default resource pack from the internet,
//! - parsing the files,
//! - loading the parsed files into the asset server,
//! - generating a new ModelBaker dynamically when loaded resource packs change.

mod blockstate;
mod model;
mod texture_mcmeta;

use std::io::{Cursor, Read};

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    platform::collections::HashMap,
    prelude::*,
};
use vec1::smallvec_v1::SmallVec1;
use zip::ZipArchive;

const DEFAULT_RESOURCE_PACK_URL: &str = "https://cdn.jsdelivr.net/gh/zdragg/minecraft-java-default-resource-pack@26.3/DefaultResourcePack.zip";

pub(super) struct InnerMinecraftAssetPlugin;

impl Plugin for InnerMinecraftAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ResourcePack>()
            .init_asset_loader::<ResourcePackLoader>()
            .add_systems(Startup, load_default_resource_pack)
            .add_systems(
                Update,
                build_model_baker.run_if(resource_changed::<ResourcePackStack>),
            );
    }
}

#[derive(Resource)]
struct ResourcePackStack(SmallVec1<[Handle<ResourcePack>; 1]>);

impl ResourcePackStack {
    fn new(base: Handle<ResourcePack>) -> Self {
        Self(SmallVec1::new(base))
    }
}

fn load_default_resource_pack(mut commands: Commands, asset_server: Res<AssetServer>) {
    let resource_pack = asset_server.load(DEFAULT_RESOURCE_PACK_URL);
    commands.insert_resource(ResourcePackStack::new(resource_pack));
}

#[derive(Asset, TypePath)]
struct ResourcePack;

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

        let mut archive = ZipArchive::new(Cursor::new(buf))?;

        let mut blockstates = HashMap::new();
        let mut models = HashMap::new();
        let mut textures = HashMap::new();
        let mut textures_mcmeta = HashMap::new();

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            if !entry.is_file() {
                continue;
            }
            let path = entry.name()?;
            let path_parts: Vec<_> = path.splitn(4, '/').collect();

            if let ["assets", namespace, "blockstates", rest] = &path_parts[..]
                && let Some(name) = rest.strip_suffix(".json")
            {
                // path = assets/minecraft/blockstates/acacia_button.json
                // namespace = minecraft
                // name = acacia_button
                let key = format!("{namespace}:{name}");

                let mut buf = Vec::with_capacity(entry.size() as usize);
                entry.read_to_end(&mut buf)?;
                let blockstate: blockstate::Blockstate =
                    serde_json::from_slice(&buf).map_err(|e| format!("blockstate {key}: {e}"))?;

                blockstates.insert(key, blockstate);
            } else if let ["assets", namespace, "models", rest] = &path_parts[..]
                && let Some(name) = rest.strip_suffix(".json")
            {
                // path = assets/minecraft/models/block/acacia_button.json
                // namespace = minecraft
                // name = block/acacia_button
                let key = format!("{namespace}:{name}");

                let mut buf = Vec::with_capacity(entry.size() as usize);
                entry.read_to_end(&mut buf)?;
                let model: model::Model =
                    serde_json::from_slice(&buf).map_err(|e| format!("model {key}: {e}"))?;

                models.insert(key, model);
            } else if let ["assets", namespace, "textures", rest] = &path_parts[..] {
                if let Some(name) = rest.strip_suffix(".png") {
                    // path = assets/minecraft/textures/block/acacia_leaves.png
                    // namespace = minecraft
                    // name = block/acacia_leaves
                    let key = format!("{namespace}:{name}");

                    let mut buf = Vec::with_capacity(entry.size() as usize);
                    entry.read_to_end(&mut buf)?;

                    textures.insert(key, buf);
                } else if let Some(name) = rest.strip_suffix(".png.mcmeta") {
                    // path = assets/minecraft/textures/block/acacia_leaves.png.mcmeta
                    // namespace = minecraft
                    // name = block/acacia_leaves
                    let key = format!("{namespace}:{name}");

                    let mut buf = Vec::with_capacity(entry.size() as usize);
                    entry.read_to_end(&mut buf)?;
                    let texture_mcmeta: texture_mcmeta::TextureMcmeta =
                        serde_json::from_slice(&buf)
                            .map_err(|e| format!("texture mcmeta {key}: {e}"))?;

                    textures_mcmeta.insert(key, texture_mcmeta);
                }
            }
        }

        Ok(ResourcePack)
    }

    fn extensions(&self) -> &[&str] {
        &["zip"]
    }
}

pub(crate) mod models {}

#[derive(Resource, PartialEq)]
struct ModelBaker;

fn build_model_baker(mut commands: Commands, resource_pack_stack: Res<ResourcePackStack>) {
    for resource_pack in &resource_pack_stack.0 {}
    commands.insert_resource_if_neq(ModelBaker);
}
