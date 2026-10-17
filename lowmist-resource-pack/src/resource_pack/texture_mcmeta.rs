//! <https://minecraft.wiki/w/Resource_pack#Texture_animation>

use std::num::NonZeroU32;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TextureMcmeta {
    /// Contains the animation properties.
    pub animation: Option<Animation>,
    /// Contains information about how to render the texture.
    pub texture: Option<Texture>,
}

#[derive(Debug, Deserialize)]
pub struct Animation {
    /// If true, the game generates additional frames between frames
    /// if the frametime is greater than 1 between them.
    /// Defaults to false.
    #[serde(default)]
    pub interpolate: bool,

    /// The width of a single frame in pixels.
    /// Defaults to the image's height if height is defined, or the
    /// smaller of the image's dimensions if it isn't.
    pub width: Option<NonZeroU32>,

    /// The height of a single frame, in pixels.
    /// Defaults to the image's width if width is defined, or the
    /// smaller of the image's dimensions if it isn't.
    pub height: Option<NonZeroU32>,

    /// Sets the default time for each frame in increments of one game tick.
    /// Defaults to 1.
    #[serde(default = "default_frametime")]
    pub frametime: NonZeroU32,

    #[serde(default)]
    /// Contains a list of frames.
    /// If not specified, all frames are displayed first to last.
    pub frames: Vec<Frame>,
}

fn default_frametime() -> NonZeroU32 {
    NonZeroU32::MIN
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Frame {
    Default(
        /// The index of a frame, with the first frame being 0.
        u32,
    ),
    TimeOverrided {
        /// The index of a frame, with the first frame being 0.
        index: u32,

        /// The time in ticks to show this frame, overriding frametime above.
        #[serde(default = "default_frametime")]
        time: NonZeroU32,
    },
}

#[derive(Debug, Deserialize)]
pub struct Texture {
    /// Causes the texture to blur when viewed from close up.
    /// Defaults to false.
    #[serde(default)]
    pub blur: bool,

    /// Causes the texture to stretch instead of tiling in cases where it otherwise would, such as on the shadow.
    /// Defaults to false.
    #[serde(default)]
    pub clamp: bool,

    #[serde(default)]
    pub mipmap_strategy: MipmapStrategy,

    #[serde(default)]
    pub alpha_cutoff_bias: f32,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MipmapStrategy {
    #[default]
    Auto,
    Mean,
    Cutout,
    StrictCutout,
    DarkCutout,
}
