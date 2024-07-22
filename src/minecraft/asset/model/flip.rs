use crate::minecraft::asset::types::rotation::StateRotation;

use super::{Axis, ElementFace, ElementFaces, Model, ModelElement};

pub trait Flip {
    fn flip(&mut self, axis: &Axis) {
        match axis {
            Axis::X => self.flip_x(),
            Axis::Y => self.flip_y(),
            Axis::Z => self.flip_z(),
        }
    }

    fn flip_x(&mut self);

    fn flip_y(&mut self);

    fn flip_z(&mut self);
}

impl Flip for Model {
    fn flip_x(&mut self) {
        self.elements.iter_mut().for_each(Flip::flip_x);
    }

    fn flip_y(&mut self) {
        self.elements.iter_mut().for_each(Flip::flip_y);
    }

    fn flip_z(&mut self) {
        self.elements.iter_mut().for_each(Flip::flip_z);
    }
}

impl Flip for ModelElement {
    fn flip_x(&mut self) {
        self.faces.flip_x();

        let from = -self.to.x + 16.0;
        self.to.x = -self.from.x + 16.0;
        self.from.x = from;
    }

    fn flip_y(&mut self) {
        self.faces.flip_y();

        let from = -self.to.y + 16.0;
        self.to.y = -self.from.y + 16.0;
        self.from.y = from;
    }

    fn flip_z(&mut self) {
        self.faces.flip_z();

        let from = -self.to.z + 16.0;
        self.to.z = -self.from.z + 16.0;
        self.from.z = from;
    }
}

impl Flip for ElementFaces {
    fn flip_x(&mut self) {
        if let Some(face) = &mut self.north {
            face.flip_uv_x();
        }
        if let Some(face) = &mut self.south {
            face.flip_uv_x();
        }
        if let Some(face) = &mut self.up {
            face.flip_uv_x();
        }
        if let Some(face) = &mut self.down {
            face.flip_uv_x();
        }

        std::mem::swap(&mut self.west, &mut self.east);

        if let Some(face) = &mut self.east {
            face.flip_uv_y();
        }
        if let Some(face) = &mut self.west {
            face.flip_uv_y();
        }
    }

    fn flip_y(&mut self) {
        if let Some(face) = &mut self.north {
            face.flip_uv_y();
        }
        if let Some(face) = &mut self.south {
            face.flip_uv_y();
        }
        if let Some(face) = &mut self.east {
            face.flip_uv_y();
        }
        if let Some(face) = &mut self.west {
            face.flip_uv_y();
        }

        std::mem::swap(&mut self.down, &mut self.up);

        if let Some(face) = &mut self.up {
            face.flip_uv_x();
        }
        if let Some(face) = &mut self.down {
            face.flip_uv_x();
        }
    }

    fn flip_z(&mut self) {
        if let Some(face) = &mut self.west {
            face.flip_uv_x();
        }
        if let Some(face) = &mut self.east {
            face.flip_uv_x();
        }
        if let Some(face) = &mut self.up {
            face.flip_uv_y();
        }
        if let Some(face) = &mut self.down {
            face.flip_uv_y();
        }

        std::mem::swap(&mut self.south, &mut self.north);

        if let Some(face) = &mut self.north {
            face.flip_uv_x();
        }
        if let Some(face) = &mut self.south {
            face.flip_uv_x();
        }
    }
}

impl ElementFace {
    fn flip_uv_x(&mut self) {
        if let Some(uv) = &mut self.uv {
            let (x1, y1, x2, y2) = uv.into();

            match self.rotation {
                StateRotation::Degrees0 | StateRotation::Degrees180 => {
                    std::mem::swap(x1, x2);
                }
                StateRotation::Degrees90 | StateRotation::Degrees270 => {
                    std::mem::swap(y1, y2);
                }
            }
        }
    }

    fn flip_uv_y(&mut self) {
        if let Some(uv) = &mut self.uv {
            let (x1, y1, x2, y2) = uv.into();

            match self.rotation {
                StateRotation::Degrees90 | StateRotation::Degrees270 => {
                    std::mem::swap(x1, x2);
                }
                StateRotation::Degrees0 | StateRotation::Degrees180 => {
                    std::mem::swap(y1, y2);
                }
            }
        }
    }
}
