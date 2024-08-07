pub mod flip;
pub mod rotate;
pub mod translate;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::minecraft::serialize::*;

use super::{
    types::{
        identifier::{AssetType, Identifier},
        rotation::StateRotation,
        variable::VariableIdentifier,
        vec::{Vec3, Vec4},
    },
    Asset,
};

#[skip_serializing_none]
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Model {
    pub parent: Option<Identifier>,
    #[serde(rename = "ambientocclusion")]
    pub ambient_occlusion: Option<bool>,
    pub display: Option<ItemDisplayPositions>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub textures: IndexMap<String, IdentifierOrVariable>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<ModelElement>,
    pub gui_light: Option<GuiLightDirection>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overrides: Vec<ItemModelOverride>,
    #[serde(default, skip_serializing)]
    pub mcpacker: ModelSettings,
}

impl Model {
    pub fn set_cullface(&mut self, value: &CullDirection) {
        for element in &mut self.elements {
            element.faces.set_cullface(value);
        }
    }

    pub fn update_texture(&mut self, old: &VariableIdentifier, new: VariableIdentifier) {
        for element in &mut self.elements {
            for face in (&mut element.faces).into_iter().flatten() {
                if face.texture == *old {
                    face.texture = new.clone();
                }
            }
        }
    }
}

impl Asset for Model {
    fn get_type() -> AssetType {
        AssetType::Model
    }

    fn is_virtual(&self) -> bool {
        self.mcpacker.is_virtual
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum IdentifierOrVariable {
    Identifier(Identifier),
    Variable(VariableIdentifier),
}

#[derive(Debug, PartialEq, Eq, Default, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CullDirection {
    #[default]
    None,
    North,
    East,
    South,
    West,
    Up,
    Down,
}

impl CullDirection {
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelElement {
    pub from: Vec3,
    pub to: Vec3,
    pub rotation: Option<ElementRotation>,
    #[serde(default = "get_true", skip_serializing_if = "is_true")]
    pub shade: bool,
    #[serde(default, skip_serializing_if = "ElementFaces::is_default")]
    pub faces: ElementFaces,
}

impl ModelElement {
    #[inline]
    pub fn north_mut(&mut self) -> (&mut f32, FaceNormal) {
        (&mut self.from.z, FaceNormal::Negative)
    }

    #[inline]
    pub fn east_mut(&mut self) -> (&mut f32, FaceNormal) {
        (&mut self.to.x, FaceNormal::Positive)
    }

    #[inline]
    pub fn south_mut(&mut self) -> (&mut f32, FaceNormal) {
        (&mut self.to.z, FaceNormal::Positive)
    }

    #[inline]
    pub fn west_mut(&mut self) -> (&mut f32, FaceNormal) {
        (&mut self.from.x, FaceNormal::Negative)
    }

    #[inline]
    pub fn up_mut(&mut self) -> (&mut f32, FaceNormal) {
        (&mut self.to.y, FaceNormal::Positive)
    }

    #[inline]
    pub fn down_mut(&mut self) -> (&mut f32, FaceNormal) {
        (&mut self.from.y, FaceNormal::Negative)
    }

    pub fn within_cube(&self, cube_from: &Vec3, cube_to: &Vec3) -> bool {
        let (from_x, from_y, from_z) = (&self.from).into();
        let (to_x, to_y, to_z) = (&self.to).into();

        *from_x >= cube_from.x
            && *from_y >= cube_from.y
            && *from_z >= cube_from.z
            && *to_x <= cube_to.x
            && *to_y <= cube_to.y
            && *to_z <= cube_to.z
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum FaceNormal {
    Positive = 1,
    Negative = -1,
}

#[skip_serializing_none]
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ElementFaces {
    pub north: Option<ElementFace>,
    pub east: Option<ElementFace>,
    pub south: Option<ElementFace>,
    pub west: Option<ElementFace>,
    pub up: Option<ElementFace>,
    pub down: Option<ElementFace>,
}

impl ElementFaces {
    pub fn set_cullface(&mut self, value: &CullDirection) {
        for face in self.into_iter().flatten() {
            face.cullface = value.clone();
        }
    }

    fn is_default(&self) -> bool {
        self.north.is_none()
            && self.east.is_none()
            && self.south.is_none()
            && self.west.is_none()
            && self.up.is_none()
            && self.down.is_none()
    }
}

impl IntoIterator for ElementFaces {
    type Item = Option<ElementFace>;
    type IntoIter = std::array::IntoIter<Self::Item, 6>;

    fn into_iter(self) -> Self::IntoIter {
        [
            self.north, self.east, self.south, self.west, self.up, self.down,
        ]
        .into_iter()
    }
}

impl<'a> IntoIterator for &'a ElementFaces {
    type Item = Option<&'a ElementFace>;
    type IntoIter = std::array::IntoIter<Self::Item, 6>;

    fn into_iter(self) -> Self::IntoIter {
        [
            self.north.as_ref(),
            self.east.as_ref(),
            self.south.as_ref(),
            self.west.as_ref(),
            self.up.as_ref(),
            self.down.as_ref(),
        ]
        .into_iter()
    }
}

impl<'a> IntoIterator for &'a mut ElementFaces {
    type Item = Option<&'a mut ElementFace>;
    type IntoIter = std::array::IntoIter<Self::Item, 6>;

    fn into_iter(self) -> Self::IntoIter {
        [
            self.north.as_mut(),
            self.east.as_mut(),
            self.south.as_mut(),
            self.west.as_mut(),
            self.up.as_mut(),
            self.down.as_mut(),
        ]
        .into_iter()
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ElementRotation {
    origin: Vec3,
    axis: Axis,
    angle: f32,
    #[serde(default, skip_serializing_if = "is_false")]
    rescale: bool,
}

impl ElementRotation {
    pub fn is_zero(&self) -> bool {
        self.angle == 0.0
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Axis {
    X,
    Y,
    Z,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ElementFace {
    pub uv: Option<Vec4>,
    pub texture: VariableIdentifier,
    #[serde(default, skip_serializing_if = "CullDirection::is_default")]
    pub cullface: CullDirection,
    #[serde(default, skip_serializing_if = "StateRotation::is_default")]
    pub rotation: StateRotation,
    #[serde(
        default = "ElementFace::default_tint_index",
        skip_serializing_if = "ElementFace::is_default_tint_index",
        rename = "tintindex"
    )]
    pub tint_index: i32,
}

impl ElementFace {
    #[inline]
    fn default_tint_index() -> i32 {
        -1
    }

    #[inline]
    fn is_default_tint_index(value: &i32) -> bool {
        *value == -1
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
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
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItemDisplay {
    rotation: Option<Vec3>,
    translation: Option<Vec3>,
    scale: Option<Vec3>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GuiLightDirection {
    Front,
    Side,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItemModelOverride {
    predicate: IndexMap<String, i32>,
    model: Identifier,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct ModelSettings {
    #[serde(rename = "virtual")]
    is_virtual: bool,
}
