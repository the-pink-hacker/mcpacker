use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TextureMeta {
    animation: TextureAnimation,
}

#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TextureAnimation {
    interpolate: Option<bool>,
    width: Option<u32>,
    height: Option<u32>,
    frametime: Option<u32>,
    frames: Option<AnimationFrames>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnimationFrames {
    Index(Vec<i32>),
    IndexTime { index: u32, time: u32 },
}
