//! <https://minecraft.wiki/w/Resource_pack#Texture_animation>

use std::num::NonZeroU32;

use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct TextureMcmeta {
    /// Contains the animation properties.
    animation: Animation,
}

#[derive(Deserialize)]
pub(crate) struct Animation {
    /// If true, the game generates additional frames between frames
    /// if the frametime is greater than 1 between them.
    /// Defaults to false.
    #[serde(default)]
    pub(crate) interpolate: bool,

    /// The width of a single frame in pixels.
    /// Defaults to the image's height if height is defined, or the
    /// smaller of the image's dimensions if it isn't.
    pub(crate) width: Option<NonZeroU32>,

    /// The height of a single frame, in pixels.
    /// Defaults to the image's width if width is defined, or the
    /// smaller of the image's dimensions if it isn't.
    pub(crate) height: Option<NonZeroU32>,

    /// Sets the default time for each frame in increments of one game tick.
    /// Defaults to 1.
    #[serde(default = "default_frametime")]
    pub(crate) frametime: NonZeroU32,

    /// Contains a list of frames.
    /// If not specified, all frames are displayed first to last.
    pub(crate) frames: Vec<Frame>,
}

fn default_frametime() -> NonZeroU32 {
    NonZeroU32::MIN
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum Frame {
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
