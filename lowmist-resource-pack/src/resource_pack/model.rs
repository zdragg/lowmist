use serde::Deserialize;

use crate::resource_pack::ResourceLocation;

#[derive(Debug, Deserialize)]
pub struct Model {
    pub parent: ResourceLocation,
}
