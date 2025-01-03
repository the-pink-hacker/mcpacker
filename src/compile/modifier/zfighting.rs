use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    asset::selector::AssetSelector,
    compile::{modifier::Modifier, PackCompiler},
    minecraft::asset::{
        model::{FaceNormal, Model, ModelElement},
        types::{identifier::Identifier, vec::Vec3},
    },
};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    North,
    East,
    South,
    West,
    Up,
    Down,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ZFightingModifier {
    pub direction: Direction,
    #[serde(default)]
    pub cube_check: bool,
    #[serde(default)]
    pub cube_from: Vec3,
    #[serde(default = "ZFightingModifier::default_cube_to")]
    pub cube_to: Vec3,
    pub offset: f32,
    #[serde(default)]
    pub random_offset: f32,
    pub intersection: f32,
    #[serde(default)]
    pub selector: AssetSelector,
}

impl ZFightingModifier {
    pub fn get_offset(&self, rand: &mut impl Rng) -> f32 {
        self.offset + rand.gen_range(-self.random_offset..=self.random_offset)
    }

    fn default_cube_to() -> Vec3 {
        Vec3::new(16.0, 16.0, 16.0)
    }
}

impl Modifier<Model, Identifier> for ZFightingModifier {
    fn apply_modifier(&self, asset: &mut Model, compiler: &mut PackCompiler) {
        let offset = self.get_offset(&mut compiler.rand);

        for element in &mut asset.elements {
            element.apply_zfighting_modifier(self, offset)
        }
    }

    fn does_modifier_apply(&self, id: &Identifier) -> bool {
        self.selector.applies(id)
    }
}

impl ModelElement {
    fn apply_zfighting_modifier(&mut self, modifier: &ZFightingModifier, offset: f32) {
        if modifier.cube_check && self.within_cube(&modifier.cube_from, &modifier.cube_to) {
            return;
        }

        let (face_position, face_normal) = match modifier.direction {
            Direction::North => self.north_mut(),
            Direction::East => self.east_mut(),
            Direction::South => self.south_mut(),
            Direction::West => self.west_mut(),
            Direction::Up => self.up_mut(),
            Direction::Down => self.down_mut(),
        };

        if *face_position == modifier.intersection {
            match face_normal {
                FaceNormal::Positive => *face_position -= offset,
                FaceNormal::Negative => *face_position += offset,
            }
        }
    }
}
