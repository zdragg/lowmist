//! <https://minecraft.wiki/w/Blockstates_definition>

use std::{collections::BTreeMap, num::NonZeroU32, str::FromStr};

use nutype::nutype;
use serde::Deserialize;
use serde_with::DeserializeFromStr;

use crate::{
    resource_pack::{HashMap, ResourceLocation},
    shared::QuarterRotation,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlockstateDispatcher {
    /// Holds the names of all the variants of the block.​
    /// ```jsonc
    /// {
    ///   "variants": {
    ///     "facing=north,lit=false": /* OneOrManyVariantsA */,
    ///     "facing=north,lit=true": /* OneOrManyVariantsB */
    ///     // ...
    ///   }
    /// }
    /// ```
    Variants(HashMap<Properties, OneOrManyVariants>),

    /// Used instead of variants to combine models based on block state attributes.
    /// ```jsonc
    /// {
    ///   "multipart": [
    ///     { /* SelectorA */ },
    ///     { /* SelectorB */ }
    ///     // ...
    ///   ]
    /// }
    /// ```
    Multipart(Vec<Selector>),
}

/// Determines a case and the model that should apply in that case.
/// ```jsonc
/// {
///   "apply": /* OneOrManyVariants */,
///   "when": /* Condition */
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct Selector {
    /// Determines the model(s) to apply and its properties.
    /// There can be one model or an array of models.
    /// If set to an array, the model is chosen randomly from the options given,
    /// with each option being specified in separate subsidiary [NBT Compound / JSON Object]-tags.
    pub apply: OneOrManyVariants,

    /// A list of cases that have to be met for the model to be applied. If unset, the model always applies.
    pub when: Option<Condition>,
}

#[derive(Debug, Deserialize)]
/// A list of cases that all have to match the block to return true.
pub enum Condition {
    /// Matches if all of the contained cases return true. Cannot be set alongside other cases.
    /// ```jsonc
    /// {
    ///   "AND": [
    ///     { /* ConditionA */ },
    ///     { /* ConditionB */ }
    ///     // ...
    ///   ]
    /// }
    /// ```
    #[serde(rename = "AND")]
    And(Vec<Condition>),

    /// Matches if any of the contained cases return true. Cannot be set alongside other cases.
    /// ```jsonc
    /// {
    ///   "OR": [
    ///     { /* ConditionA */ },
    ///     { /* ConditionB */ }
    ///     // ...
    ///   ]
    /// }
    /// ```
    #[serde(rename = "OR")]
    Or(Vec<Condition>),

    /// Cannot be set alongside the OR or AND tag.
    /// ```jsonc
    /// {
    ///   "facing": "north",
    ///   "flower_amount": "2|3|4"
    ///   // ...
    /// }
    /// ```
    #[serde(untagged)]
    Properties(HashMap<String, Terms>),
}

/// ```jsonc
/// "!north|south"
/// ```
#[derive(Debug, DeserializeFromStr)]
pub struct Terms {
    pub entries: Vec<Term>,
}

impl FromStr for Terms {
    type Err = !;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let terms: Vec<Term> = s.split('|').map(|s| s.parse().into_ok()).collect();
        Ok(Terms { entries: terms })
    }
}

/// ```jsonc
/// "!north"
/// ```
#[derive(Debug)]
pub struct Term {
    pub value: String,
    pub negated: bool,
}

impl FromStr for Term {
    type Err = !;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let term = match s.strip_prefix('!') {
            Some(trimmed) => Term {
                value: trimmed.to_owned(),
                negated: true,
            },
            None => Term {
                value: s.to_owned(),
                negated: false,
            },
        };
        Ok(term)
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OneOrManyVariants {
    /// ```jsonc
    /// {
    ///   /* Variant */
    /// }
    /// ```
    One(Variant),

    /// ```jsonc
    /// [
    ///   { /* WeightedVariantA */ },
    ///   { /* WeightedVariantB */ }
    ///   // ...
    /// ]
    /// ```
    Many(WeightedVariants),
}

/// ```jsonc
/// {
///   "model": "minecraft:block/acacia_button",
///   "x": 90,
///   "y": 180,
///   "uvlock": true
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct Variant {
    /// Specifies the path to the model file of the block, in form of a resource location.
    pub model: ResourceLocation,

    /// Optional. Default: 0.
    /// Rotation of the model on the x-axis in increments of 90 degrees.
    #[serde(default)]
    pub x: QuarterRotation,

    /// Optional. Default: 0.
    /// Rotation of the model on the y-axis in increments of 90 degrees.
    #[serde(default)]
    pub y: QuarterRotation,

    /// Optional. Default: 0.
    /// Rotation of the model on the z-axis in increments of 90 degrees.
    #[serde(default)]
    pub z: QuarterRotation,

    /// Optional. Default: false.
    /// If set to true, locks the rotation of the texture of a block.
    /// This way the texture does not rotate with the block when using the x and y tags above.
    #[serde(default)]
    pub uvlock: bool,
}

#[nutype(derive(Deserialize, Debug), validate(predicate = |variants| !variants.is_empty()))]
pub struct WeightedVariants(Vec<WeightedVariant>);

/// ```jsonc
/// {
///   "model": "minecraft:block/chorus_plant_noside",
///   "y": 90,
///   "weight": 2
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct WeightedVariant {
    #[serde(flatten)]
    pub variant: Variant,

    /// Optional. Default: 1.
    /// If more than one model is used for the same variant, the probability is
    /// calculated by dividing the individual model's weight by the sum of
    /// the weights of all models.
    /// (For example, if three models are used with weights 1, 1, and 2,
    /// then their combined weight would be 4 (1+1+2). The probability of
    /// each model being used would then be determined by dividing
    /// each weight by 4: 1/4, 1/4 and 2/4, or 25%, 25% and 50%, respectively.)
    #[serde(default = "default_weight")]
    pub weight: NonZeroU32,
}

fn default_weight() -> NonZeroU32 {
    NonZeroU32::MIN
}

/// "facing=north" matches ALL blockstates with "facing=north", including:
///
/// "facing=north" \
/// "facing=north,lit=true" \
/// "facing=north,whatever=whatever"
///
/// ```jsonc
/// "facing=north,lit=false"
/// ```
#[derive(Debug, DeserializeFromStr, Hash, PartialEq, Eq)]
pub struct Properties(pub BTreeMap<String, String>);

impl FromStr for Properties {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self(BTreeMap::new()));
        }
        let map = s
            .split(',')
            .map(|term| {
                term.split_once('=')
                    .map(|(a, b)| (a.to_owned(), b.to_owned()))
                    .ok_or("invalid blockstate")
            })
            .collect::<Result<_, _>>()?;
        Ok(Self(map))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_wall_torch() {
        let json = r#"
            {
                "variants": {
                    "facing=east": { "model": "block/wall_torch" },
                    "facing=south": { "model": "block/wall_torch", "y": 90 },
                    "facing=west": { "model": "block/wall_torch", "y": 180 },
                    "facing=north": { "model": "block/wall_torch", "y": 270 }
                }
            }
            "#;

        let _: BlockstateDispatcher = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn parse_grass_block() {
        let json = r#"
            {
                "variants": {
                    "snowy=false": [
                        { "model": "block/grass_block" },
                        { "model": "block/grass_block", "y": 90 },
                        { "model": "block/grass_block", "y": 180 },
                        { "model": "block/grass_block", "y": 270 }
                    ],
                    "snowy=true":  { "model": "block/grass_block_snow" }
                }
            }
            "#;

        let _: BlockstateDispatcher = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn parse_oak_fence() {
        let json = r#"
            {
                "multipart": [
                    {   "apply": { "model": "block/oak_fence_post" }},
                    {   "when": { "north": "true" },
                        "apply": { "model": "block/oak_fence_side", "uvlock": true }
                    },
                    {   "when": { "east": "true" },
                        "apply": { "model": "block/oak_fence_side", "y": 90, "uvlock": true }
                    },
                    {   "when": { "south": "true" },
                        "apply": { "model": "block/oak_fence_side", "y": 180, "uvlock": true }
                    },
                    {   "when": { "west": "true" },
                        "apply": { "model": "block/oak_fence_side", "y": 270, "uvlock": true }
                    }
                ]
            }
            "#;

        let _: BlockstateDispatcher = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn parse_redstone_wire() {
        let json = r#"
            {
                "multipart": [
                    {   "when": { "OR": [
                            {"north": "none", "east": "none", "south": "none", "west": "none"},
                            {"north": "side|up", "east": "side|up" },
                            {"east": "side|up", "south": "side|up" },
                            {"south": "side|up", "west": "side|up"},
                            {"west": "side|up", "north": "side|up"}
                        ]},
                        "apply": { "model": "block/redstone_dust_dot" }
                    },
                    {   "when": { "OR": [
                            { "north": "side|up" },
                            { "north": "none", "east": "none", "south": "side|up", "west": "none" }
                        ]},
                        "apply": { "model": "block/redstone_dust_side0" }
                    },
                    {   "when": { "OR": [
                            { "south": "side|up" },
                            { "north": "side|up", "east": "none", "south": "none", "west": "none" }
                        ]},
                        "apply": { "model": "block/redstone_dust_side_alt0" }
                    },
                    {   "when": { "OR": [
                            { "east": "side|up" },
                            { "north": "none", "east": "none", "south": "none", "west": "side|up" }
                        ]},
                        "apply": { "model": "block/redstone_dust_side_alt1", "y": 270 }
                    },
                    {   "when": { "OR": [
                            { "west": "side|up" },
                            { "north": "none", "east": "side|up", "south": "none", "west": "none" }
                        ]},
                        "apply": { "model": "block/redstone_dust_side1", "y": 270 }
                    },
                    {   "when": { "north": "up" },
                        "apply": { "model": "block/redstone_dust_up" }
                    },
                    {   "when": { "east": "up" },
                        "apply": { "model": "block/redstone_dust_up", "y": 90 }
                    },
                    {   "when": { "south": "up" },
                        "apply": { "model": "block/redstone_dust_up", "y": 180 }
                    },
                    {   "when": { "west": "up" },
                        "apply": { "model": "block/redstone_dust_up", "y": 270 }
                    }
                ]
            }
            "#;

        let _: BlockstateDispatcher = serde_json::from_str(json).unwrap();
    }

    #[test]
    #[should_panic]
    fn parse_empty_many_variants() {
        let json = r#"
            {
                "variants": {
                    "snowy=false": [],
                    "snowy=true":  { "model": "block/grass_block_snow" }
                }
            }
            "#;

        let _: BlockstateDispatcher = serde_json::from_str(json).unwrap();
    }
}
