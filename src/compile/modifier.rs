use crate::minecraft::asset::{model::Model, types::identifier::Identifier};

use self::culling::CullingModifier;

use super::{library::CompiledAssetLibrary, PackCompiler};

pub mod configurable;
pub mod culling;
pub mod redirect;
pub mod zfighting;

pub type ModelModifiers = Vec<Box<dyn Modifier<Model, Identifier> + Send + Sync>>;

pub trait Modifier<A, S> {
    fn apply_modifier(&self, asset: &mut A, compiler: &mut PackCompiler);

    fn does_modifier_apply(&self, _id: &S) -> bool {
        true
    }
}

impl<'a> PackCompiler<'a> {
    pub fn process_modifiers(&mut self, library: &mut CompiledAssetLibrary) -> anyhow::Result<()> {
        library.modifiers.push(Box::new(CullingModifier::default()));

        library.apply_model_modifiers(self);

        Ok(())
    }
}

impl CompiledAssetLibrary {
    fn apply_model_modifiers(&mut self, compiler: &mut PackCompiler) {
        self.models.iter_mut().for_each(|(model_id, model)| {
            self.modifiers
                .iter()
                .filter(|modifier| modifier.does_modifier_apply(model_id))
                .for_each(|modifier| modifier.apply_modifier(model, compiler))
        });
    }
}
