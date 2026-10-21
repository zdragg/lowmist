//! <https://minecraft.wiki/w/Model>

use std::{collections::BTreeMap, str::FromStr};

use nutype::nutype;
use serde::Deserialize;
use serde_inline_default::serde_inline_default;
use serde_with::DeserializeFromStr;

use crate::{resource_pack::ResourceLocation, shared::QuarterRotation};

#[serde_inline_default]
#[derive(Debug, Deserialize)]
pub struct Model {
    /// Loads a different model from the given path, in form of
    /// a resource location.
    ///
    /// If both "parent" and "elements" are set, the "elements"
    /// tag overrides the "elements" tag from the previous model.
    pub parent: Option<ResourceLocation>,

    /// Whether to use ambient occlusion.
    ///
    /// Default: true
    #[serde_inline_default(true)]
    pub ambientocclusion: bool,

    /// Holds the textures of the model, in form of
    /// a resource location or another texture variable.
    pub textures: Option<BTreeMap<TextureVariable, Texture>>,

    /// Contains all the elements of the model.
    ///
    /// They can have only cubic forms. If both "parent"
    /// and "elements" are set, the "elements" tag overrides
    /// the "elements" tag from the previous model.
    ///
    /// An empty array is valid and clears the "elements" tag.
    pub elements: Option<Vec<Element>>,

    /// Holds the different places where item models are displayed.
    pub display: Option<Display>,
}

impl Model {
    pub const MISSING: Model = Model {
        parent: None,
        ambientocclusion: true,
        textures: None,
        elements: None,
        display: None,
    };
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Texture {
    /// Defines a texture variable and assigns a texture.
    /// The texture can either be a reference to another texture
    /// variable, or the resource location of a texture in
    /// the textures folder.
    ///
    /// The variable can be referenced by prefixing it with a # in the model,
    /// e.g. the variable "texture" is referenced as "#texture".
    ///
    /// The texture variable named "particle" defines what texture the
    /// breaking particles are loaded from, but can also be referenced by
    /// the model as "#particle".
    ///
    /// The particle texture is also used as an overlay if you are in a
    /// nether portal, for water and lava's still textures, and for the
    /// particle textures of non-model blocks (such as for barriers).
    ///
    /// ```json
    /// "textures": {
    ///    "particle": "block/anvil",
    ///    "body": "block/anvil"
    /// },
    /// ```
    Basic(VariableOrResource),

    /// ```json
    /// "textures": {
    ///   "all": {
    ///     "sprite": "minecraft:block/light_gray_stained_glass",
    ///     "force_translucent": true
    ///   }
    /// }
    /// ```
    WithTranslucency(TextureResourceLocation),
}

#[derive(Debug, Deserialize)]
pub struct TextureResourceLocation {
    /// The resource location of a texture.
    /// Note that texture variables cannot be used here.
    pub sprite: ResourceLocation,

    /// Defines whether the texture should be considered
    /// translucent even when it is not.
    ///
    /// Normally, Minecraft determines whether a block is
    /// translucent by checking if any of its textures has
    /// any translucent pixels. Setting this to true overrides
    /// that behavior for this texture.
    #[serde(default)]
    pub force_translucent: bool,
}

impl From<ResourceLocation> for TextureResourceLocation {
    fn from(sprite: ResourceLocation) -> Self {
        Self {
            sprite,
            force_translucent: false,
        }
    }
}

#[derive(Debug, DeserializeFromStr)]
pub enum VariableOrResource {
    TextureVariable(TextureVariable),
    ResourceLocation(TextureResourceLocation),
}

impl FromStr for VariableOrResource {
    type Err = !;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = if s.starts_with('#') {
            s.parse().map(Self::TextureVariable)
        } else {
            s.parse::<ResourceLocation>()
                .map(|id| Self::ResourceLocation(id.into()))
        };

        Ok(res.into_ok())
    }
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, DeserializeFromStr)]
pub struct TextureVariable(pub String);

impl FromStr for TextureVariable {
    type Err = !;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TextureVariable(s.trim_prefix('#').to_owned()))
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Element {
    /// Start point of a cuboid according to the scheme [x, y, z].
    /// Values must be between -16 and 32.
    pub from: [ElementCoord; 3],

    /// Stop point of a cuboid according to the scheme [x, y, z].
    /// Values must be between -16 and 32.
    pub to: [ElementCoord; 3],

    /// Holds all the faces of the cuboid.
    /// If a face is left out, it does not render.
    pub faces: BTreeMap<FaceDirection, Face>,

    /// Defines the minimum light level that the element can receive.
    /// Can be 0-15, defaults to 0.
    #[serde(default)]
    pub light_emission: i32,

    /// Defines whether the faces of this element should be shaded
    /// according to their orientation (default), or should always
    /// be shaded as if they had the specified orientation.
    ///
    /// This does not affect ambient occlusion or shading based on light level.
    pub shade_direction_override: Option<FaceDirection>,
}

#[nutype(
    validate(greater_or_equal = (-16.0), less_or_equal = 32.0, finite),
    derive(Deserialize, Debug, PartialEq)
)]
pub struct ElementCoord(f32);

#[derive(Debug, Deserialize, Hash, PartialOrd, Ord, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FaceDirection {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

#[serde_inline_default]
#[derive(Debug, Deserialize, PartialEq)]
pub struct Face {
    /// Defines the area of the texture to use according to
    /// the scheme [x1, y1, x2, y2].
    ///
    /// The texture behavior is inconsistent if UV extends
    /// below 0 or above 16.
    ///
    /// If the numbers of x1 and x2 are swapped (e.g. from
    /// 0, 0, 16, 16 to 16, 0, 0, 16), the texture flips.
    ///
    /// UV is optional, and if not supplied it automatically
    /// generates based on the element's position.
    pub uv: Option<[f32; 4]>,

    /// Specifies the texture in form of the texture variable.
    pub texture: TextureVariable,

    /// Rotates the texture clockwise by the specified number of degrees.
    ///
    /// Rotation does not affect which part of the texture is used. Instead,
    /// it amounts to permutation of the selected texture vertexes
    /// (selected implicitly, or explicitly though uv).
    #[serde(default)]
    pub rotation: QuarterRotation,

    /// Specifies whether a face does not need to be rendered
    /// when there is a block touching it in the specified position.
    pub cullface: Option<FaceDirection>,

    #[serde_inline_default(-1)]
    pub tintindex: i32,
}

#[derive(Debug, Deserialize)]
pub struct Display {
    pub thirdperson_righthand: Option<ItemTransform>,
    pub thirdperson_lefthand: Option<ItemTransform>,
    pub firstperson_lefthand: Option<ItemTransform>,
    pub gui: Option<ItemTransform>,
    pub head: Option<ItemTransform>,
    pub ground: Option<ItemTransform>,
    pub fixed: Option<ItemTransform>,
    pub on_shelf: Option<ItemTransform>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ItemTransform {
    rotation: [f32; 3],
    translation: [f32; 3],
    scale: [f32; 3],
}

impl Default for ItemTransform {
    fn default() -> Self {
        Self {
            rotation: [0., 0., 0.],
            translation: [0., 0., 0.],
            scale: [1., 1., 1.],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_heavy_core() {
        let json = r##"
            {
              "display": {
                "gui": {
                  "rotation": [ 30, 225, 0 ],
                  "translation": [ 0, 3, 0],
                  "scale":[ 1, 1, 1 ]
                },
                "ground": {
                  "rotation": [ 0, 0, 0 ],
                  "translation": [ 0, 3, 0],
                  "scale":[ 0.5, 0.5, 0.5 ]
                },
                "fixed": {
                  "rotation": [ 0, 180, 0 ],
                  "translation": [ 0, 4, 0],
                  "scale":[ 1, 1, 1 ]
                },
                "thirdperson_righthand": {
                  "rotation": [ 45, 45, 0 ],
                  "translation": [ 0, 3, 0 ],
                  "scale": [ 0.5, 0.5, 0.5 ]
                }
              },
              "texture_size": [16, 16],
              "textures": {
                "all": "block/heavy_core",
                "particle": "block/heavy_core"
              },
              "elements": [
                {
                  "name": "heavy_core",
                  "from": [4, 0, 4],
                  "to": [12, 8, 12],
                  "faces": {
                    "north": {"uv": [0, 8, 8, 16], "texture": "all"},
                    "east": {"uv": [0, 8, 8, 16], "texture": "all"},
                    "south": {"uv": [0, 8, 8, 16], "texture": "all"},
                    "west": {"uv": [0, 8, 8, 16], "texture": "all"},
                    "up": {"uv": [0, 0, 8, 8], "texture": "all"},
                    "down": {"uv": [8, 0, 16, 8], "texture": "all"}
                  }
                }
              ]
            }
            "##;

        let _: Model = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn parse_template_anvil() {
        let json = r##"
            {   "parent": "block/block",
                "textures": {
                    "particle": "block/anvil",
                    "body": "block/anvil"
                },
                "display": {
                    "fixed": {
                        "rotation": [ 0, 90, 0 ],
                        "translation": [ 0, 0, 0 ],
                        "scale": [ 0.5, 0.5, 0.5 ]
                    },
                    "on_shelf": {
                      "rotation": [ 0, 90, 0 ],
                      "translation": [ 0, 0, 0 ],
                      "scale": [ 1, 1, 1 ]
                    }
                },
                "elements": [
                    {   "__comment": "Anvil base",
                        "from": [ 2, 0, 2 ],
                        "to": [ 14, 4, 14 ],
                        "faces": {
                            "down":  { "uv": [ 2,  2, 14, 14 ], "texture": "#body", "rotation": 180, "cullface": "down" },
                            "up":    { "uv": [ 2,  2, 14, 14 ], "texture": "#body", "rotation": 180 },
                            "north": { "uv": [ 2, 12, 14, 16 ], "texture": "#body" },
                            "south": { "uv": [ 2, 12, 14, 16 ], "texture": "#body" },
                            "west":  { "uv": [ 0,  2,  4, 14 ], "texture": "#body", "rotation":  90 },
                            "east":  { "uv": [ 4,  2,  0, 14 ], "texture": "#body", "rotation": 270 }
                        }
                    },
                    {   "__comment": "Lower narrow portion",
                        "from": [ 4, 4, 3 ],
                        "to": [ 12, 5, 13 ],
                        "faces": {
                            "up":    { "uv": [ 4,  3, 12, 13 ], "texture": "#body", "rotation": 180 },
                            "north": { "uv": [ 4, 11, 12, 12 ], "texture": "#body" },
                            "south": { "uv": [ 4, 11, 12, 12 ], "texture": "#body" },
                            "west":  { "uv": [ 4,  3,  5, 13 ], "texture": "#body", "rotation":  90 },
                            "east":  { "uv": [ 5,  3,  4, 13 ], "texture": "#body", "rotation": 270 }
                        }
                    },
                    {   "__comment": "Wider section beneath top portion",
                        "from": [ 6, 5, 4 ],
                        "to": [ 10, 10, 12 ],
                        "faces": {
                            "north": { "uv": [  6,  6, 10, 11 ], "texture": "#body" },
                            "south": { "uv": [  6,  6, 10, 11 ], "texture": "#body" },
                            "west":  { "uv": [  5,  4, 10, 12 ], "texture": "#body", "rotation":  90 },
                            "east":  { "uv": [ 10,  4,  5, 12 ], "texture": "#body", "rotation": 270 }
                        }
                    },
                    {   "__comment": "Anvil top",
                        "from": [ 3, 10, 0 ],
                        "to": [ 13, 16, 16 ],
                        "faces": {
                            "down":  { "uv": [  3, 0, 13, 16 ], "texture": "#body", "rotation": 180 },
                            "up":    { "uv": [  3, 0, 13, 16 ], "texture": "#top",   "rotation": 180 },
                            "north": { "uv": [  3, 0, 13,  6 ], "texture": "#body" },
                            "south": { "uv": [  3, 0, 13,  6 ], "texture": "#body" },
                            "west":  { "uv": [ 10, 0, 16, 16 ], "texture": "#body", "rotation":  90 },
                            "east":  { "uv": [ 16, 0, 10, 16 ], "texture": "#body", "rotation": 270 }
                        }
                    }
                ]
            }
            "##;

        let _: Model = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn parse_light_gray_stained_glass() {
        let json = r##"
            {
              "parent": "minecraft:block/cube_all",
              "textures": {
                "all": {
                  "force_translucent": true,
                  "sprite": "minecraft:block/light_gray_stained_glass"
                }
              }
            }
            "##;

        let _: Model = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn parse_template_torch() {
        let json = r##"
        {
           "ambientocclusion": false,
           "textures": {
               "particle": "#torch"
           },
           "elements": [
               {   "from": [ 7, 0, 7 ],
                   "to": [ 9, 10, 9 ],
                   "shade": false,
                   "faces": {
                       "down": { "uv": [ 7, 13, 9, 15 ], "texture": "#torch" },
                       "up":   { "uv": [ 7,  6, 9,  8 ], "texture": "#torch" }
                   }
               },
               {   "from": [ 7, 0, 0 ],
                   "to": [ 9, 16, 16 ],
                   "shade": false,
                   "faces": {
                       "west": { "uv": [ 0, 0, 16, 16 ], "texture": "#torch" },
                       "east": { "uv": [ 0, 0, 16, 16 ], "texture": "#torch" }
                   }
               },
               {   "from": [ 0, 0, 7 ],
                   "to": [ 16, 16, 9 ],
                   "shade": false,
                   "faces": {
                       "north": { "uv": [ 0, 0, 16, 16 ], "texture": "#torch" },
                       "south": { "uv": [ 0, 0, 16, 16 ], "texture": "#torch" }
                   }
               }
           ]
        }
        "##;

        let _: Model = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn parse_any_block() {
        let json = r##"
            {
              "elements": [
                {
                  "from": [ 0, 0, 0 ],
                  "to": [ 16, 16, 16 ],
                  "faces": {
                    "down":  { "texture": "#down", "cullface": "down" },
                    "up":    { "texture": "#up", "cullface": "up" },
                    "north": { "texture": "#north", "cullface": "north" },
                    "south": { "texture": "#south", "cullface": "south" },
                    "west":  { "texture": "#west", "cullface": "west" },
                    "east":  { "texture": "#east", "cullface": "east" }
                  }
                }
              ]
            }
            "##;

        let _: Model = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn parse_sapling() {
        let json = r##"
            {
              "ambientocclusion": false,
              "textures": {
                "particle": "#cross"
              },
              "elements": [
                {
                  "from": [ 0.8, 0, 8 ],
                  "to": [ 15.2, 16, 8 ],
                  "rotation": { "origin": [ 8, 8, 8 ], "axis": "y", "angle": 45, "rescale": true },
                  "shade": false,
                  "faces": {
                    "north": { "uv": [ 0, 0, 16, 16 ], "texture": "#cross" },
                    "south": { "uv": [ 0, 0, 16, 16 ], "texture": "#cross" }
                  }
                },
                {
                  "from": [ 8, 0, 0.8 ],
                  "to": [ 8, 16, 15.2 ],
                  "rotation": { "origin": [ 8, 8, 8 ], "axis": "y", "angle": 45, "rescale": true },
                  "shade": false,
                  "faces": {
                    "west": { "uv": [ 0, 0, 16, 16 ], "texture": "#cross" },
                    "east": { "uv": [ 0, 0, 16, 16 ], "texture": "#cross" }
                  }
                }
              ]
            }
            "##;

        let _: Model = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn parse_fletching_table() {
        let json = r##"
            {
                "parent": "block/cube",
                "textures": {
                    "particle": "block/fletching_table",
                    "all": "block/fletching_table",
                    "bottom": "block/birch_planks"
                },
                "elements": [
                    {
                       "from": [ 0, 0, 0 ],
                        "to": [ 16, 16, 16 ],
                        "faces": {
                            "down":  { "texture": "#bottom", "cullface": "down" },
                            "up":    { "uv": [0,0, 16,5.34], "texture": "#all", "cullface": "up" },
                            "north": { "uv": [0,5.35, 16,10.66], "texture": "#all", "cullface": "north" },
                            "south": { "uv": [0,5.35, 16,10.66], "texture": "#all", "cullface": "south" },
                            "west":  { "uv": [0,10.67, 16,16], "texture": "#all", "cullface": "west" },
                            "east":  { "uv": [0,10.67, 16,16], "texture": "#all", "cullface": "east" }
                        }
                    }
                ]
            }
            "##;

        let _: Model = serde_json::from_str(json).unwrap();
    }
}
