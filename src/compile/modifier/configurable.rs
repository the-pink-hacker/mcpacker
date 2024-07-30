use serde::Deserialize;
use serde_with::{serde_as, OneOrMany};

use crate::{
    asset::LoadableAsset,
    minecraft::asset::{
        model::Model,
        types::identifier::{AssetType, Identifier},
        Asset,
    },
};

use super::{redirect::Redirect, zfighting::ZFightingModifier, Modifier};

#[serde_as]
#[derive(Debug, Default, Deserialize)]
pub struct ConfigurableModifierIndex {
    #[serde_as(as = "OneOrMany<_>")]
    pub order: Vec<Identifier>,
}

impl Asset for ConfigurableModifierIndex {
    fn get_type() -> AssetType {
        AssetType::ModifierIndex
    }
}

impl LoadableAsset for ConfigurableModifierIndex {
    fn load_asset<R: AsRef<str>>(raw: R) -> anyhow::Result<Self> {
        Ok(toml::from_str(raw.as_ref())?)
    }
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct ConfigurableModifierCollection {
    #[serde_as(as = "OneOrMany<_>")]
    pub modifiers: Vec<ConfigurableModifier>,
}

impl Asset for ConfigurableModifierCollection {
    fn get_type() -> AssetType {
        AssetType::Modifier
    }
}

impl LoadableAsset for ConfigurableModifierCollection {
    fn load_asset<R: AsRef<str>>(raw: R) -> anyhow::Result<Self> {
        Ok(toml::from_str(raw.as_ref())?)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConfigurableModifier {
    ZFighting(Box<ZFightingModifier>),
    Redirect(Box<Redirect>),
}

impl From<ConfigurableModifier> for Box<dyn Modifier<Model, Identifier> + Send + Sync> {
    fn from(value: ConfigurableModifier) -> Self {
        match value {
            ConfigurableModifier::ZFighting(modifier) => modifier,
            ConfigurableModifier::Redirect(modifier) => modifier,
        }
    }
}
