use indexmap::IndexMap;
use serde::Deserialize;

use crate::minecraft::asset::{
    model::Model,
    types::{identifier::Identifier, rotation::StateRotation, variable::VariableIdentifier},
};

#[derive(Debug, Deserialize)]
pub struct ModelPreprocessed {
    #[serde(flatten)]
    model: Model,
    #[serde(default)]
    import: IndexMap<String, Identifier>,
    #[serde(default)]
    composition: Vec<ModelComposition>,
}

#[derive(Debug, Deserialize)]
pub struct ModelComposition {
    model: VariableIdentifier,
    #[serde(default)]
    x: StateRotation,
    #[serde(default)]
    y: StateRotation,
}
