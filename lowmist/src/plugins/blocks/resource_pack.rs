//! Minecraft Resource Pack handler.
//!
//! - downloads the default resource pack from the internet
//! - parses the files using `lowmist-respack`
//! - loads the parsed files into the asset server
//! - generates a new ModelBaker dynamically when loaded resource packs change

use std::{collections::BTreeMap, str::FromStr};

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use lowmist_respack::{
    ResourceLocation,
    blockstate::{BlockstateDispatcher, OneOrManyVariants},
    model::{
        self, Element, Model, Texture, TextureResourceLocation, TextureVariable, VariableOrResource,
    },
    texture::TextureMcmeta,
};
use nunny::NonEmpty;

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
    ) -> Result<Option<()>> {
        let id = ResourceLocation::from_str(block_name).into_ok();
        let variants = self.get_blockstate(&id).find_map(|blockstate_dispatcher| {
            // Does this blockstate dispatcher have definitions for `block_properties`?
            blockstate_dispatcher.dispatch(block_properties)
        });
        let Some(variants) = variants else {
            return Ok(None);
        };

        let variant_iter = variants.into_iter().map(|variant| match variant {
            OneOrManyVariants::Many(many) => &many.first().variant, // TODO: make quads for each weighted variant
            OneOrManyVariants::One(one) => one,
        });

        for variant in variant_iter {
            let Some(resolved_model) =
                ResolvedModel::resolve_model_chain(&variant.model, |id| self.get_model(id))?
            else {
                continue;
            };
        }
        Ok(Some(()))
    }
}

struct ResolvedModel<'a> {
    elements: &'a NonEmpty<[Element]>,
    textures: HashMap<&'a TextureVariable, &'a TextureResourceLocation>,
}

impl<'a> ResolvedModel<'a> {
    fn resolve_model_chain(
        tail: &'a ResourceLocation,
        get_model: impl Fn(&'a ResourceLocation) -> Option<&'a Model>,
    ) -> Result<Option<Self>, BevyError> {
        let mut elements: Option<&[Element]> = None;
        let mut current_id: Option<&ResourceLocation> = Some(tail);
        let mut seen_ids: HashSet<&ResourceLocation> = HashSet::new();
        let mut unresolved_map: HashMap<&TextureVariable, &model::Texture> = HashMap::new();

        while let Some(id) = current_id {
            if !seen_ids.insert(id) {
                // seen
                Err(format!("model {tail} has cyclic dependency"))?;
            }
            let model = get_model(id).ok_or(format!(
                "model {tail} has invalid model {id} in parent chain"
            ))?;
            if let Some(textures) = &model.textures {
                for (k, v) in textures {
                    // Don't overwrite.
                    unresolved_map.entry(k).or_insert(v);
                }
            }
            // First found element array is used
            if let Some(this_element) = &model.elements
                && elements == None
            {
                elements = Some(this_element);
            }
            current_id = model.parent.as_ref();
        }

        // Whole chain does not define elements, nothing to render
        let Some(elements) = elements else {
            return Ok(None);
        };

        // Elements is empty, nothing to render
        let Some(elements) = NonEmpty::<[Element]>::new(elements) else {
            return Ok(None);
        };

        let used_texture_variables: HashSet<_> = elements
            .iter()
            .flat_map(|elem| elem.faces.values().map(|face| &face.texture))
            .collect();

        let mut resolved_map: HashMap<&TextureVariable, &TextureResourceLocation> = HashMap::new();
        for var in used_texture_variables {
            if resolved_map.contains_key(var) {
                // already resolved
                continue;
            }

            let mut current_var = var;
            let mut seen_vars: HashSet<&TextureVariable> = HashSet::new();

            let id = loop {
                if !seen_vars.insert(current_var) {
                    // seen
                    Err(format!(
                        "model {tail} has cyclic texture variable definition"
                    ))?;
                }
                let texture = unresolved_map.get(current_var).ok_or(format!(
                    "model {tail} has unmapped texture variable definition"
                ))?;

                match *texture {
                    Texture::Basic(VariableOrResource::ResourceLocation(id))
                    | Texture::WithTranslucency(id) => break id,

                    Texture::Basic(VariableOrResource::TextureVariable(var)) => {
                        if let Some(id) = resolved_map.get(var).cloned() {
                            break id;
                        }
                        current_var = var;
                    }
                }
            };

            for seen_var in seen_vars {
                resolved_map.insert(seen_var, id);
            }
        }

        Ok(Some(Self {
            elements,
            textures: resolved_map,
        }))
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

        let quads = pack_stack.get_quads(block_name, block_properties)?;
    }

    commands.insert_resource_if_neq(ModelBaker);
    Ok(())
}
