use std::collections::HashMap;

use anyhow::Context;
use serde::Deserialize;

use crate::minecraft::asset::{
    model::{rotate::Rotatable, translate::Translate, CullDirection, Model},
    types::{
        identifier::{AssetType, Identifier},
        rotation::StateRotation,
        variable::VariableIdentifier,
        vec::Vec3,
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
    transformations: Vec<Transformation>,
    cullface: Option<CullDirection>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Transformation {
    Rotate {
        #[serde(default)]
        x: StateRotation,
        #[serde(default)]
        y: StateRotation,
    },
    Translate {
        amount: Vec3,
    },
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

        if let Some(cullface) = &self.cullface {
            lookup_model.set_cullface(cullface);
        }

        for transform in &self.transformations {
            match transform {
                Transformation::Rotate { x, y } => {
                    lookup_model.rotate_x(x);
                    lookup_model.rotate_y(y);
                }
                Transformation::Translate { amount } => lookup_model.translate(amount),
            }
        }

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
