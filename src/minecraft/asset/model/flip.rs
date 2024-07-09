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

        let east = self.west.clone();
        self.west = self.east.clone();
        self.east = east;

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

        let up = self.down.clone();
        self.down = self.up.clone();
        self.up = up;

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

        let north = self.south.clone();
        self.south = self.north.clone();
        self.north = north;

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
                    let x2_new = *x1;
                    *x1 = *x2;
                    *x2 = x2_new;
                }
                StateRotation::Degrees90 | StateRotation::Degrees270 => {
                    let y2_new = *y1;
                    *y1 = *y2;
                    *y2 = y2_new;
                }
            }
        }
    }

    fn flip_uv_y(&mut self) {
        if let Some(uv) = &mut self.uv {
            let (x1, y1, x2, y2) = uv.into();

            match self.rotation {
                StateRotation::Degrees90 | StateRotation::Degrees270 => {
                    let x2_new = *x1;
                    *x1 = *x2;
                    *x2 = x2_new;
                }
                StateRotation::Degrees0 | StateRotation::Degrees180 => {
                    let y2_new = *y1;
                    *y1 = *y2;
                    *y2 = y2_new;
                }
            }
        }
    }
}
