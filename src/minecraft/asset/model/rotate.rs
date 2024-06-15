use crate::minecraft::asset::types::rotation::StateRotation;

use super::{CullDirection, ElementFaces, Model, ModelElement};

pub trait Rotatable {
    fn rotate_x(&mut self, amount: &StateRotation) {
        for _ in 0..amount.quarters() {
            self.rotate_x_quarter()
        }
    }

    fn rotate_y(&mut self, amount: &StateRotation) {
        for _ in 0..amount.quarters() {
            self.rotate_y_quarter()
        }
    }

    fn rotate_x_quarter(&mut self);

    fn rotate_y_quarter(&mut self);
}

impl Rotatable for Model {
    fn rotate_x_quarter(&mut self) {
        for element in &mut self.elements {
            element.rotate_x_quarter();
        }
    }

    fn rotate_y_quarter(&mut self) {
        for element in &mut self.elements {
            element.rotate_y_quarter();
        }
    }
}

impl Rotatable for CullDirection {
    fn rotate_x_quarter(&mut self) {
        match self {
            Self::North => *self = Self::Up,
            Self::South => *self = Self::Down,
            Self::Up => *self = Self::South,
            Self::Down => *self = Self::North,
            _ => (),
        }
    }

    fn rotate_y_quarter(&mut self) {
        match self {
            Self::North => *self = Self::West,
            Self::East => *self = Self::North,
            Self::South => *self = Self::East,
            Self::West => *self = Self::South,
            _ => (),
        }
    }
}

impl Rotatable for ModelElement {
    fn rotate_x_quarter(&mut self) {
        self.faces.rotate_x_quarter();

        let (_, from_y, from_z) = self.from.into_tuple_mut();
        let (_, to_y, to_z) = self.to.into_tuple_mut();

        let length_y = *to_z - *from_z;
        let length_z = *to_y - *from_y;

        let corner_y = -*to_z + 16.0;
        let corner_z = *from_y;

        *from_y = corner_y;
        *from_z = corner_z;
        *to_y = corner_y + length_y;
        *to_z = corner_z + length_z;
    }

    fn rotate_y_quarter(&mut self) {
        self.faces.rotate_y_quarter();

        let (from_x, _, from_z) = self.from.into_tuple_mut();
        let (to_x, _, to_z) = self.to.into_tuple_mut();

        let length_x = *to_z - *from_z;
        let length_z = *to_x - *from_x;

        let corner_x = *from_z;
        let corner_z = -*to_x + 16.0;

        *from_x = corner_x;
        *from_z = corner_z;
        *to_x = corner_x + length_x;
        *to_z = corner_z + length_z;
    }
}

impl Rotatable for ElementFaces {
    fn rotate_x_quarter(&mut self) {
        let temp = self.down.clone();
        self.down = self.south.clone();
        self.south = self.up.clone();
        self.up = self.north.clone();
        self.north = temp;

        if let Some(face) = &mut self.north {
            if face.uv.is_some() {
                face.rotation = face.rotation.opposite();
            }
            face.cullface.rotate_x_quarter();
        }

        if let Some(face) = &mut self.east {
            if face.uv.is_some() {
                face.rotation = face.rotation.rotate_quarter_counter();
            }
            face.cullface.rotate_x_quarter();
        }

        if let Some(face) = &mut self.south {
            face.cullface.rotate_x_quarter();
        }

        if let Some(face) = &mut self.west {
            if face.uv.is_some() {
                face.rotation = face.rotation.rotate_quarter();
            }

            face.cullface.rotate_x_quarter();
        }

        if let Some(face) = &mut self.up {
            if face.uv.is_some() {
                face.rotation = face.rotation.opposite();
            }
            face.cullface.rotate_x_quarter();
        }

        if let Some(face) = &mut self.down {
            face.cullface.rotate_x_quarter();
        }
    }

    fn rotate_y_quarter(&mut self) {
        let temp = self.west.clone();
        self.west = self.north.clone();
        self.north = self.east.clone();
        self.east = self.south.clone();
        self.south = temp;

        if let Some(face) = &mut self.north {
            face.cullface.rotate_y_quarter();
        }

        if let Some(face) = &mut self.east {
            face.cullface.rotate_y_quarter();
        }

        if let Some(face) = &mut self.south {
            face.cullface.rotate_y_quarter();
        }

        if let Some(face) = &mut self.west {
            face.cullface.rotate_y_quarter();
        }

        if let Some(face) = &mut self.up {
            if face.uv.is_some() {
                face.rotation = face.rotation.rotate_quarter_counter();
            }
            face.cullface.rotate_y_quarter();
        }

        if let Some(face) = &mut self.down {
            if face.uv.is_some() {
                face.rotation = face.rotation.rotate_quarter();
            }
            face.cullface.rotate_y_quarter();
        }
    }
}

impl StateRotation {
    pub fn rotate_quarter(&self) -> Self {
        match self {
            Self::Degrees0 => Self::Degrees90,
            Self::Degrees90 => Self::Degrees180,
            Self::Degrees180 => Self::Degrees270,
            Self::Degrees270 => Self::Degrees0,
        }
    }

    pub fn rotate_quarter_counter(&self) -> Self {
        match self {
            Self::Degrees0 => Self::Degrees270,
            Self::Degrees90 => Self::Degrees0,
            Self::Degrees180 => Self::Degrees90,
            Self::Degrees270 => Self::Degrees180,
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Self::Degrees0 => Self::Degrees180,
            Self::Degrees90 => Self::Degrees270,
            Self::Degrees180 => Self::Degrees0,
            Self::Degrees270 => Self::Degrees90,
        }
    }
}
