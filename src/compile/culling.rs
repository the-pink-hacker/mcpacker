use crate::minecraft::asset::{
    model::{Model, ModelElement},
    types::identifier::Identifier,
};

use super::{modifier::Modifier, PackCompiler};

#[derive(Default)]
pub struct CullingModifier;

impl Modifier<Model, Identifier> for CullingModifier {
    fn apply_modifier(&self, asset: &mut Model, _compiler: &mut PackCompiler) {
        let elements_length = asset.elements.len();

        // One single element is never both mutable and imutable.
        // Borrow checker doesn't know this.
        // More idomatic way would be better.
        let scan_assets = asset.elements.clone();

        for primary_index in (0..elements_length).rev() {
            let primary_cube = asset.elements.get_mut(primary_index).unwrap();

            for scan_index in 0..elements_length {
                if scan_index == primary_index {
                    continue;
                }

                let scan_cube = scan_assets.get(scan_index).unwrap();

                let element_empty = primary_cube.cull_faces(scan_cube);

                if element_empty {
                    asset.elements.swap_remove(primary_index);
                    break;
                }
            }
        }
    }
}

impl ModelElement {
    // Returns `true` when all faces are empty.
    fn cull_faces(&mut self, other: &Self) -> bool {
        let (from_x, from_y, from_z) = (&self.from).into();
        let (to_x, to_y, to_z) = (&self.to).into();

        let (other_from_x, other_from_y, other_from_z) = (&other.from).into();
        let (other_to_x, other_to_y, other_to_z) = (&other.to).into();

        let within_x = from_x >= other_from_x && to_x <= other_to_x;
        let within_y = from_y >= other_from_y && to_y <= other_to_y;
        let within_z = from_z >= other_from_z && to_z <= other_to_z;

        let mut empty_faces = 0u8;

        if self.faces.north.is_some()
            && other.faces.south.is_some()
            && within_x
            && within_y
            && from_z == other_to_z
        {
            self.faces.north.take();
            empty_faces += 1;
        }

        if self.faces.east.is_some()
            && other.faces.west.is_some()
            && within_z
            && within_y
            && to_x == other_from_x
        {
            self.faces.east.take();
            empty_faces += 1;
        }

        if self.faces.south.is_some()
            && other.faces.north.is_some()
            && within_x
            && within_y
            && to_z == other_from_z
        {
            self.faces.south.take();
            empty_faces += 1;
        }

        if self.faces.west.is_some()
            && other.faces.east.is_some()
            && within_z
            && within_y
            && from_x == other_to_x
        {
            self.faces.west.take();
            empty_faces += 1;
        }

        if self.faces.up.is_some()
            && other.faces.down.is_some()
            && within_x
            && within_z
            && to_y == other_from_y
        {
            self.faces.up.take();
            empty_faces += 1;
        }

        if self.faces.down.is_some()
            && other.faces.up.is_some()
            && within_x
            && within_z
            && from_y == other_to_y
        {
            self.faces.down.take();
            empty_faces += 1;
        }

        // All faces have been culled.
        empty_faces == 6
    }
}
