use serde::Deserialize;

use crate::{
    asset::selector::AssetSelector,
    compile::PackCompiler,
    minecraft::asset::{
        model::{IdentifierOrVariable, Model},
        types::identifier::{AssetType, Identifier},
    },
};

use super::Modifier;

#[derive(Debug, Deserialize, Clone)]
pub struct Redirect {
    pub affect: RedirectAffect,
    pub asset_type: AssetType,
    pub from: Identifier,
    pub to: Identifier,
    #[serde(default)]
    pub selector: AssetSelector,
}

impl Modifier<Model, Identifier> for Redirect {
    fn apply_modifier(&self, asset: &mut Model, _compiler: &mut PackCompiler) {
        for texture in asset.textures.values_mut() {
            match texture {
                IdentifierOrVariable::Variable(_) => (),
                IdentifierOrVariable::Identifier(id) => id.apply_redirect(self),
            }
        }
    }

    fn does_modifier_apply(&self, id: &Identifier) -> bool {
        self.selector.applies(id)
    }
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
            self.namespace.clone_from(&redirect.to.namespace);
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
            selector: Default::default(),
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
            selector: Default::default(),
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
            selector: Default::default(),
        };
        let mut id = Identifier::minecraft("blocks/dirt");
        id.apply_redirect(&redirect);
        let expected = Identifier::new("quark", "block/dirt");

        assert_eq!(id, expected);
    }
}
