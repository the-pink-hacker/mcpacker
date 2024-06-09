use serde::Deserialize;

use crate::minecraft::asset::{
    model::{IdentifierOrVariable, Model},
    types::identifier::{AssetType, Identifier},
};

#[derive(Debug, Deserialize, Clone)]
pub struct Redirect {
    pub affect: RedirectAffect,
    pub asset_type: AssetType,
    pub from: Identifier,
    pub to: Identifier,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum RedirectAffect {
    Path,
}

impl Identifier {
    pub fn apply_redirect(&mut self, redirect: &Redirect) {
        match redirect.affect {
            RedirectAffect::Path => self.apply_redirect_path(redirect),
        }
    }

    fn apply_redirect_path(&mut self, redirect: &Redirect) {
        if self.namespace != redirect.from.namespace {
            return;
        }

        if let Ok(path) = self.path.strip_prefix(&redirect.from.path) {
            self.path = redirect.to.path.join(path);
            self.namespace = redirect.to.namespace.clone();
        }
    }
}

impl Model {
    pub fn apply_texture_redirect(&mut self, redirect: &Redirect) {
        if let Some(ref mut textures) = &mut self.textures {
            for texture in textures.values_mut() {
                match texture {
                    IdentifierOrVariable::Variable(_) => (),
                    IdentifierOrVariable::Identifier(id) => id.apply_redirect(redirect),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redirect_path() {
        let redirect = Redirect {
            affect: RedirectAffect::Path,
            asset_type: AssetType::Texture,
            from: Identifier::minecraft("blocks"),
            to: Identifier::minecraft("block"),
        };
        let mut id = Identifier::minecraft("blocks/dirt");
        id.apply_redirect(&redirect);
        let expected = Identifier::minecraft("block/dirt");

        assert_eq!(id, expected);
    }

    #[test]
    fn redirect_path_unaffected() {
        let redirect = Redirect {
            affect: RedirectAffect::Path,
            asset_type: AssetType::Texture,
            from: Identifier::minecraft("items"),
            to: Identifier::minecraft("item"),
        };
        let mut id = Identifier::minecraft("blocks/dirt");
        let expected = id.clone();
        id.apply_redirect(&redirect);

        assert_eq!(id, expected);
    }

    #[test]
    fn redirect_path_namespace() {
        let redirect = Redirect {
            affect: RedirectAffect::Path,
            asset_type: AssetType::Texture,
            from: Identifier::minecraft("blocks"),
            to: Identifier::new("quark", "block"),
        };
        let mut id = Identifier::minecraft("blocks/dirt");
        id.apply_redirect(&redirect);
        let expected = Identifier::new("quark", "block/dirt");

        assert_eq!(id, expected);
    }
}
