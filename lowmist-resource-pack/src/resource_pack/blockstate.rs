//! <https://minecraft.wiki/w/Blockstates_definition>

use std::{num::NonZeroU32, str::FromStr};

use serde::Deserialize;
use serde_repr::Deserialize_repr;
use serde_with::DeserializeFromStr;

use crate::resource_pack::{HashMap, ResourceLocation};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Blockstate {
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
    Variants(HashMap<String, OneOrManyVariants>),

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
    Many(Vec<WeightedVariant>),
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
    pub x: Rotation,

    /// Optional. Default: 0.
    /// Rotation of the model on the y-axis in increments of 90 degrees.
    #[serde(default)]
    pub y: Rotation,

    /// Optional. Default: 0.
    /// Rotation of the model on the z-axis in increments of 90 degrees.
    #[serde(default)]
    pub z: Rotation,

    /// Optional. Default: false.
    /// If set to true, locks the rotation of the texture of a block.
    /// This way the texture does not rotate with the block when using the x and y tags above.
    #[serde(default)]
    pub uvlock: bool,
}

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

#[derive(Debug, Default, Deserialize_repr)]
#[repr(u16)]
pub enum Rotation {
    #[default]
    R0 = 0,
    R90 = 90,
    R180 = 180,
    R270 = 270,
}
