//! Minecraft Resource Pack handler.
//!
//! - downloads the default resource pack from the internet
//! - parses the files using `lowmist-respack`
//! - loads the parsed files into the asset server
//! - generates a new ModelBaker dynamically when loaded resource packs change

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};

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

#[derive(Resource, PartialEq)]
struct ModelBaker;

fn build_model_baker(mut commands: Commands, resource_pack_stack: Res<ResourcePackStack>) {
    for resource_pack in &resource_pack_stack.0 {}
    commands.insert_resource_if_neq(ModelBaker);
}
