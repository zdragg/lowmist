pub mod blockstate;
pub mod model;
pub mod shared;
pub mod texture;

use std::{
    fmt::Display,
    io::{Cursor, Read, Seek},
    str::FromStr,
};

use itertools::Itertools;
use serde::de::DeserializeOwned;
use serde_with::DeserializeFromStr;
use snafu::{ResultExt, Snafu};
use zip::{ZipArchive, read::ZipFile, result::ZipError};

use crate::resource_pack::{
    blockstate::BlockstateDispatcher, model::Model, texture::TextureMcmeta,
};

type HashMap<K, V> = std::collections::HashMap<K, V, foldhash::fast::FixedState>;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to open zip archive"))]
    OpenZip { source: ZipError },

    #[snafu(display("Failed to access zip entry {index}"))]
    GetZipEntry { source: ZipError, index: usize },

    #[snafu(display("Zip entry {index} has an invalid path"))]
    InvalidZipEntryName { source: ZipError, index: usize },

    #[snafu(display("Failed to read zip entry {entry_name}"))]
    ReadZipEntry {
        source: std::io::Error,
        entry_name: String,
    },

    #[snafu(display("Failed to parse JSON under {key_kind} {key}"))]
    ParseJson {
        source: serde_json::Error,
        key_kind: String,
        key: ResourceLocation,
    },
}

#[derive(Debug)]
pub struct ResourcePack {
    pub blockstates: HashMap<ResourceLocation, BlockstateDispatcher>,
    pub models: HashMap<ResourceLocation, Model>,
    pub textures: HashMap<ResourceLocation, Vec<u8>>,
    pub textures_mcmeta: HashMap<ResourceLocation, TextureMcmeta>,
}

impl ResourcePack {
    pub fn load(bytes: &[u8]) -> Result<ResourcePack, Error> {
        let mut archive = ZipArchive::new(Cursor::new(bytes)).context(OpenZipSnafu)?;

        let mut blockstates: HashMap<ResourceLocation, BlockstateDispatcher> = HashMap::default();
        let mut models: HashMap<ResourceLocation, Model> = HashMap::default();
        let mut textures: HashMap<ResourceLocation, Vec<u8>> = HashMap::default();
        let mut textures_mcmeta: HashMap<ResourceLocation, TextureMcmeta> = HashMap::default();

        for index in 0..archive.len() {
            let mut entry = archive
                .by_index(index)
                .context(GetZipEntrySnafu { index })?;
            if !entry.is_file() {
                continue;
            }
            let entry_name = entry
                .name()
                .context(InvalidZipEntryNameSnafu { index })?
                .into_owned();
            let Some(("assets", namespace, kind, rest)) = entry_name.splitn(4, '/').collect_tuple()
            else {
                continue;
            };

            match kind {
                "blockstates" => {
                    if let Some(path) = rest.strip_suffix(".json") {
                        let (key, blockstate) = parse_zip_entry(namespace, path, &mut entry)?;
                        blockstates.insert(key, blockstate);
                    }
                }
                "models" => {
                    if let Some(path) = rest.strip_suffix(".json") {
                        let (key, model) = parse_zip_entry(namespace, path, &mut entry)?;
                        models.insert(key, model);
                    }
                }
                "textures" => {
                    if let Some(path) = rest.strip_suffix(".png.mcmeta") {
                        let (key, texture_mcmeta) = parse_zip_entry(namespace, path, &mut entry)?;
                        textures_mcmeta.insert(key, texture_mcmeta);
                    } else if let Some(path) = rest.strip_suffix(".png") {
                        let (key, bytes) = read_zip_entry(namespace, path, &mut entry)?;
                        textures.insert(key, bytes);
                    }
                }
                _ => {}
            }
        }

        Ok(ResourcePack {
            blockstates,
            models,
            textures,
            textures_mcmeta,
        })
    }
}

fn read_zip_entry<R: Read + Seek>(
    namespace: &str,
    path: &str,
    entry: &mut ZipFile<'_, R>,
) -> Result<(ResourceLocation, Vec<u8>), Error> {
    let key = ResourceLocation::new(namespace, path);
    let mut buf = Vec::with_capacity(entry.size() as usize);
    entry.read_to_end(&mut buf).context(ReadZipEntrySnafu {
        entry_name: entry
            .name()
            .expect("entry name successfully decoded before"),
    })?;
    Ok((key, buf))
}

fn parse_zip_entry<R: Read + Seek, T: DeserializeOwned>(
    namespace: &str,
    path: &str,
    entry: &mut ZipFile<'_, R>,
) -> Result<(ResourceLocation, T), Error> {
    let (key, buf) = read_zip_entry(namespace, path, entry)?;
    let result = serde_json::from_slice(&buf).context(ParseJsonSnafu {
        key_kind: std::any::type_name::<T>(),
        key: key.clone(),
    })?;
    Ok((key, result))
}

#[derive(Clone, Debug, DeserializeFromStr, Hash, PartialEq, Eq)]
pub struct ResourceLocation {
    pub namespace: String,
    pub path: String,
}

impl ResourceLocation {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }
}

impl FromStr for ResourceLocation {
    type Err = !;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = match s.split_once(":") {
            Some((namespace, path)) => ResourceLocation {
                namespace: namespace.into(),
                path: path.into(),
            },
            None => ResourceLocation {
                namespace: "minecraft".into(),
                path: s.into(),
            },
        };
        Ok(result)
    }
}

impl Display for ResourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}
