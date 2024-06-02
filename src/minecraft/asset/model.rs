use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::types::{identifier::Identifier, rotation::StateRotation};

#[derive(Debug, Deserialize, Serialize)]
pub struct Model {
    parent: Option<Identifier>,
    #[serde(rename = "ambient_occlusion")]
    ambient_occlusion: Option<bool>,
    display: Option<ItemDisplayPositions>,
    textures: Option<IndexMap<String, IdentifierOrVariable>>,
    elements: Option<Vec<ModelElement>>,
    gui_light: Option<GuiLightDirection>,
    overrides: Option<Vec<ItemModelOverride>>,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelElement {
    from: [f32; 3],
    to: [f32; 3],
    rotation: Option<ElementRotation>,
    shade: Option<bool>,
    faces: Option<ElementFaces>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ElementFaces {
    north: Option<ElementFace>,
    east: Option<ElementFace>,
    south: Option<ElementFace>,
    west: Option<ElementFace>,
    up: Option<ElementFace>,
    down: Option<ElementFace>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ElementRotation {
    origin: [f32; 3],
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

#[derive(Debug, Deserialize, Serialize)]
pub struct ElementFace {
    uv: Option<[f32; 4]>,
    texture: VariableIdentifier,
    cullface: Option<Direction>,
    rotation: Option<StateRotation>,
    #[serde(rename = "tintindex")]
    tint_index: Option<i32>,
}

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

#[derive(Debug, Deserialize, Serialize)]
pub struct ItemDisplay {
    rotation: Option<[f32; 3]>,
    translation: Option<[f32; 3]>,
    scale: Option<[f32; 3]>,
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