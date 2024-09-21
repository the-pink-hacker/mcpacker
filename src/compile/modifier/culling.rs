use serde::Deserialize;

use crate::{
    asset::selector::AssetSelector,
    compile::PackCompiler,
    minecraft::asset::{
        model::{Model, ModelElement},
        types::identifier::Identifier,
    },
};

use super::Modifier;

#[derive(Debug, Default, Deserialize, Clone)]
pub struct CullingModifier {
    #[serde(default)]
    selector: AssetSelector,
}

impl Modifier<Model, Identifier> for CullingModifier {
    fn apply_modifier(&self, asset: &mut Model, _compiler: &mut PackCompiler) {
        asset.scan_filter_elements(|primary_element, scan_element| {
            primary_element.cull_faces(scan_element)
        });
    }

    fn does_modifier_apply(&self, id: &Identifier) -> bool {
        self.selector.applies(id)
    }
}

impl ModelElement {
    // Returns `true` when all faces are empty.
    fn cull_faces(&mut self, other: &Self) -> bool {
        let is_rotation_zero = if let Some(rotation) = &self.rotation {
            rotation.is_zero()
        } else {
            true
        };

        if is_rotation_zero {
            let (from_x, from_y, from_z) = (&self.from).into();
            let (to_x, to_y, to_z) = (&self.to).into();

            let (other_from_x, other_from_y, other_from_z) = (&other.from).into();
            let (other_to_x, other_to_y, other_to_z) = (&other.to).into();

            let within_x = from_x >= other_from_x && to_x <= other_to_x;
            let within_y = from_y >= other_from_y && to_y <= other_to_y;
            let within_z = from_z >= other_from_z && to_z <= other_to_z;

            if self.faces.north.is_some()
                && other.faces.south.is_some()
                && within_x
                && within_y
                && from_z == other_to_z
            {
                self.faces.north.take();
            }

            if self.faces.east.is_some()
                && other.faces.west.is_some()
                && within_z
                && within_y
                && to_x == other_from_x
            {
                self.faces.east.take();
            }

            if self.faces.south.is_some()
                && other.faces.north.is_some()
                && within_x
                && within_y
                && to_z == other_from_z
            {
                self.faces.south.take();
            }

            if self.faces.west.is_some()
                && other.faces.east.is_some()
                && within_z
                && within_y
                && from_x == other_to_x
            {
                self.faces.west.take();
            }

            if self.faces.up.is_some()
                && other.faces.down.is_some()
                && within_x
                && within_z
                && to_y == other_from_y
            {
                self.faces.up.take();
            }

            if self.faces.down.is_some()
                && other.faces.up.is_some()
                && within_x
                && within_z
                && from_y == other_to_y
            {
                self.faces.down.take();
            }
        }

        // All faces have been culled.
        (&self.faces).into_iter().all(|face| face.is_none())
    }
}
