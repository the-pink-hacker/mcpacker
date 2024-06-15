use crate::minecraft::asset::types::vec::Vec3;

use super::{Model, ModelElement};

pub trait Translate {
    fn translate(&mut self, amount: &Vec3);
}

impl Translate for Model {
    fn translate(&mut self, amount: &Vec3) {
        for element in &mut self.elements {
            element.translate(amount);
        }
    }
}

impl Translate for ModelElement {
    fn translate(&mut self, amount: &Vec3) {
        self.from += amount.clone();
        self.to += amount.clone();
    }
}
