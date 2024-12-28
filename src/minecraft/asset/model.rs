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
    pub fn set_cullface(&mut self, cullface: CullDirection) {
        self.elements
            .iter_mut()
            .for_each(|element| element.set_cullface(cullface));
    }

    pub fn auto_cullface(&mut self) {
        self.elements
            .iter_mut()
            .for_each(ModelElement::auto_cullface);
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

    pub fn scan_filter_elements(
        &mut self,
        predicate: impl Fn(&mut ModelElement, &ModelElement) -> bool,
    ) {
        let mut elements_length = self.elements.len();

        // One single element is never both mutable and imutable.
        // Borrow checker doesn't know this.
        // More idomatic way would be better.
        let scan_assets = self.elements.clone();

        // Reverse to allow swap remove
        for primary_index in (0..elements_length).rev() {
            let primary_cube = self.elements.get_mut(primary_index).unwrap();

            for scan_index in 0..elements_length {
                if scan_index == primary_index {
                    continue;
                }

                let scan_cube = scan_assets.get(scan_index).unwrap();

                let element_empty = predicate(primary_cube, scan_cube);

                if element_empty {
                    self.elements.swap_remove(primary_index);
                    elements_length -= 1;
                    break;
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

#[derive(Debug, PartialEq, Eq, Default, Clone, Copy, Deserialize, Serialize)]
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
        let (from_x, from_y, from_z) = self.from.into();
        let (to_x, to_y, to_z) = self.to.into();

        from_x >= cube_from.x
            && from_y >= cube_from.y
            && from_z >= cube_from.z
            && to_x <= cube_to.x
            && to_y <= cube_to.y
            && to_z <= cube_to.z
    }

    pub fn auto_cullface(&mut self) {
        if let Some(face) = &mut self.faces.north {
            let (face_from, face_to) = &Self::north_position(self.from, self.to);
            face.cullface = Self::determine_auto_cullface(face_from, face_to);
        }

        if let Some(face) = &mut self.faces.east {
            let (face_from, face_to) = &Self::east_position(self.from, self.to);
            face.cullface = Self::determine_auto_cullface(face_from, face_to);
        }

        if let Some(face) = &mut self.faces.south {
            let (face_from, face_to) = &Self::south_position(self.from, self.to);
            face.cullface = Self::determine_auto_cullface(face_from, face_to);
        }

        if let Some(face) = &mut self.faces.west {
            let (face_from, face_to) = &Self::west_position(self.from, self.to);
            face.cullface = Self::determine_auto_cullface(face_from, face_to);
        }

        if let Some(face) = &mut self.faces.up {
            let (face_from, face_to) = &Self::up_position(self.from, self.to);
            face.cullface = Self::determine_auto_cullface(face_from, face_to);
        }

        if let Some(face) = &mut self.faces.down {
            let (face_from, face_to) = &Self::down_position(self.from, self.to);
            face.cullface = Self::determine_auto_cullface(face_from, face_to);
        }
    }

    pub fn set_cullface(&mut self, cullface: CullDirection) {
        (&mut self.faces)
            .into_iter()
            .flatten()
            .for_each(|face| face.cullface = cullface);
    }

    fn determine_auto_cullface(face_from: &Vec3, face_to: &Vec3) -> CullDirection {
        if Self::is_north_contained(face_from, face_to) {
            CullDirection::North
        } else if Self::is_east_contained(face_from, face_to) {
            CullDirection::East
        } else if Self::is_south_contained(face_from, face_to) {
            CullDirection::South
        } else if Self::is_west_contained(face_from, face_to) {
            CullDirection::West
        } else if Self::is_up_contained(face_from, face_to) {
            CullDirection::Up
        } else if Self::is_down_contained(face_from, face_to) {
            CullDirection::Down
        } else {
            CullDirection::None
        }
    }

    #[inline]
    fn north_position(from: Vec3, to: Vec3) -> (Vec3, Vec3) {
        (from, Vec3::new(to.x, to.y, from.z))
    }

    #[inline]
    fn east_position(from: Vec3, to: Vec3) -> (Vec3, Vec3) {
        (Vec3::new(to.x, from.y, from.z), to)
    }

    #[inline]
    fn south_position(from: Vec3, to: Vec3) -> (Vec3, Vec3) {
        (Vec3::new(from.x, from.y, to.z), to)
    }

    #[inline]
    fn west_position(from: Vec3, to: Vec3) -> (Vec3, Vec3) {
        (from, Vec3::new(from.x, to.y, to.z))
    }

    #[inline]
    fn up_position(from: Vec3, to: Vec3) -> (Vec3, Vec3) {
        (Vec3::new(from.x, to.y, from.z), to)
    }

    #[inline]
    fn down_position(from: Vec3, to: Vec3) -> (Vec3, Vec3) {
        (from, Vec3::new(to.x, from.y, to.z))
    }

    fn is_north_contained(from: &Vec3, to: &Vec3) -> bool {
        from.x >= 0.0 && to.x <= 16.0 && from.y >= 0.0 && to.y <= 16.0 && to.z <= 0.0
    }

    fn is_east_contained(from: &Vec3, to: &Vec3) -> bool {
        from.z >= 0.0 && to.z <= 16.0 && from.y >= 0.0 && to.y <= 16.0 && from.x >= 16.0
    }

    fn is_south_contained(from: &Vec3, to: &Vec3) -> bool {
        from.x >= 0.0 && to.x <= 16.0 && from.y >= 0.0 && to.y <= 16.0 && from.z >= 16.0
    }

    fn is_west_contained(from: &Vec3, to: &Vec3) -> bool {
        from.z >= 0.0 && to.z <= 16.0 && from.y >= 0.0 && to.y <= 16.0 && to.x <= 0.0
    }

    fn is_up_contained(from: &Vec3, to: &Vec3) -> bool {
        from.x >= 0.0 && to.x <= 16.0 && from.z >= 0.0 && to.z <= 16.0 && from.y >= 16.0
    }

    fn is_down_contained(from: &Vec3, to: &Vec3) -> bool {
        from.x >= 0.0 && to.x <= 16.0 && from.z >= 0.0 && to.z <= 16.0 && to.y <= 0.0
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
            face.cullface = *value;
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
