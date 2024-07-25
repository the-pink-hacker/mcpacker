use crate::{
    config::RedirectFile,
    minecraft::asset::{model::Model, types::identifier::Identifier},
};

use super::{library::CompiledAssetLibrary, PackCompiler};

type ModelModifiers = Vec<Box<dyn Modifier<Model, Identifier> + Send>>;

pub trait Modifier<A, S> {
    fn apply_modifier(&self, asset: &mut A, compiler: &mut PackCompiler);

    fn does_modifier_apply(&self, _id: &S) -> bool {
        true
    }
}

impl<'a> PackCompiler<'a> {
    pub async fn process_modifiers(
        &mut self,
        library: &mut CompiledAssetLibrary,
    ) -> anyhow::Result<()> {
        let mut modifiers = Vec::new();
        self.add_redirect_modifiers(&mut modifiers).await?;
        self.add_zfighting_modifiers(&mut modifiers);

        library.apply_model_modifiers(modifiers, self);

        Ok(())
    }

    async fn add_redirect_modifiers(&self, modifiers: &mut ModelModifiers) -> anyhow::Result<()> {
        modifiers.reserve(self.redirects.len());

        for redirect_path in &self.redirects {
            let raw_redirect =
                async_fs::read_to_string(self.get_redirect_path(redirect_path)?).await?;
            let redirect = toml::from_str::<RedirectFile>(&raw_redirect)?.redirect;
            modifiers.push(Box::new(redirect));
        }

        Ok(())
    }

    fn add_zfighting_modifiers(&self, modifiers: &mut ModelModifiers) {
        if let Some(zfighting_modifiers) = &self.pack.zfighting_modifiers {
            modifiers.reserve(zfighting_modifiers.len());

            for zfighting_modifier in zfighting_modifiers {
                modifiers.push(Box::new(zfighting_modifier.clone()));
            }
        }
    }
}
