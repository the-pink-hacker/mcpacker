use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{
    types::{
        identifier::{AssetType, Identifier},
        rotation::StateRotation,
        vec::{Vec3, Vec4},
    },
    Asset,
};

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct Model {
    pub parent: Option<Identifier>,
    #[serde(rename = "ambient_occlusion")]
    pub ambient_occlusion: Option<bool>,
    pub display: Option<ItemDisplayPositions>,
    pub textures: Option<IndexMap<String, IdentifierOrVariable>>,
    pub elements: Option<Vec<ModelElement>>,
    pub gui_light: Option<GuiLightDirection>,
    pub overrides: Option<Vec<ItemModelOverride>>,
}

impl Asset for Model {
    fn get_type() -> AssetType {
        AssetType::Model
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VariableIdentifier(String);

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum IdentifierOrVariable {
    Identifier(Identifier),
    Variable(VariableIdentifier),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    North,
    East,
    South,
    West,
    Up,
    Down,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct ModelElement {
    from: Vec3,
    to: Vec3,
    rotation: Option<ElementRotation>,
    shade: Option<bool>,
    faces: Option<ElementFaces>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct ElementFaces {
    north: Option<ElementFace>,
    east: Option<ElementFace>,
    south: Option<ElementFace>,
    west: Option<ElementFace>,
    up: Option<ElementFace>,
    down: Option<ElementFace>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct ElementRotation {
    origin: Vec3,
    axis: Axis,
    angle: f32,
    rescale: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Axis {
    X,
    Y,
    Z,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct ElementFace {
    uv: Option<Vec4>,
    texture: VariableIdentifier,
    cullface: Option<Direction>,
    rotation: Option<StateRotation>,
    #[serde(rename = "tintindex")]
    tint_index: Option<i32>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct ItemDisplayPositions {
    thirdperson_righthand: Option<ItemDisplay>,
    thirdperson_lefthand: Option<ItemDisplay>,
    firstperson_righthand: Option<ItemDisplay>,
    firstperson_lefthand: Option<ItemDisplay>,
    gui: Option<ItemDisplay>,
    head: Option<ItemDisplay>,
    ground: Option<ItemDisplay>,
    fixed: Option<ItemDisplay>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct ItemDisplay {
    rotation: Option<Vec3>,
    translation: Option<Vec3>,
    scale: Option<Vec3>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GuiLightDirection {
    Front,
    Side,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ItemModelOverride {
    predicate: IndexMap<String, i32>,
    model: Identifier,
}
