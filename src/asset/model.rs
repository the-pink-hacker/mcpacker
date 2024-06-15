use std::collections::HashMap;

use anyhow::Context;
use serde::Deserialize;

use crate::minecraft::asset::{
    model::{rotate::Rotatable, Model},
    types::{
        identifier::{AssetType, Identifier},
        rotation::StateRotation,
        variable::VariableIdentifier,
    },
    Asset,
};

#[derive(Debug, Deserialize)]
pub struct ModelPreprocessed {
    #[serde(flatten)]
    model: Model,
    #[serde(default)]
    import: HashMap<String, Identifier>,
    #[serde(default)]
    composition: Vec<ModelComposition>,
}

impl Asset for ModelPreprocessed {
    fn get_type() -> AssetType {
        AssetType::Model
    }
}

#[derive(Debug, Deserialize)]
pub struct ModelComposition {
    model: VariableIdentifier,
    #[serde(default)]
    x: StateRotation,
    #[serde(default)]
    y: StateRotation,
}

impl ModelComposition {
    pub fn compile(
        &self,
        model: &mut Model,
        models: &HashMap<Identifier, Model>,
        variables: &HashMap<String, Identifier>,
    ) -> anyhow::Result<()> {
        let lookup_id = variables
            .get(&self.model.clone().get_name())
            .with_context(|| format!("Failed to locate model variable: {}", self.model))?;
        let mut lookup_model = models
            .get(lookup_id)
            .with_context(|| format!("Failed to locate model: {}", lookup_id))?
            .clone();

        lookup_model.rotate_x(&self.x);
        lookup_model.rotate_y(&self.y);

        for element in &lookup_model.elements {
            model.elements.push(element.clone());
        }

        Ok(())
    }
}

impl ModelPreprocessed {
    pub fn compile(&self, models: &HashMap<Identifier, Model>) -> anyhow::Result<Model> {
        let mut model = self.model.clone();

        for ref mut part in &self.composition {
            part.compile(&mut model, models, &self.import)?;
        }

        Ok(model)
    }
}
