use fastnbt::IntArray;
use serde::Deserialize;

/// Reference: <https://github.com/sakura-ryoko/litematica/blob/f7ac844c8134745cd89a6d9690cf3c753fe57465/src/main/java/fi/dy/masa/litematica/schematic/SchematicMetadata.java#L391>
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Metadata {
    pub name: String,
    pub author: String,
    pub description: String,

    pub region_count: i32,
    pub total_volume: i32,
    pub total_blocks: i32,

    #[serde(with = "jiff::fmt::serde::timestamp::millisecond::required")]
    pub time_modified: jiff::Timestamp,
    #[serde(with = "jiff::fmt::serde::timestamp::millisecond::required")]
    pub time_created: jiff::Timestamp,

    #[serde(with = "crate::serde::ivec3::xyz")]
    pub enclosing_size: glam::IVec3,

    pub preview_image_data: Option<IntArray>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            name: "?".into(),
            author: "?".into(),
            description: "".into(),

            region_count: 0,
            total_volume: 0,
            total_blocks: 0,

            time_modified: jiff::Timestamp::from_millisecond(-1).unwrap(),
            time_created: jiff::Timestamp::from_millisecond(-1).unwrap(),

            enclosing_size: glam::IVec3::ZERO,

            preview_image_data: None,
        }
    }
}
