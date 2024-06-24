use std::collections::HashMap;

use anyhow::Context;
use indexmap::IndexMap;
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

use super::LoadableAsset;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ModelGeneric {
    Preprocessed(ModelPreprocessed),
    Normal(Model),
}

impl LoadableAsset for ModelGeneric {
    fn load_asset<R: AsRef<str>>(raw: R) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(raw.as_ref())?)
    }
}

#[derive(Debug, Deserialize)]
pub struct ModelPreprocessed {
    #[serde(default)]
    pub import: HashMap<String, ModelOrId>,
    pub composition: Vec<ModelComposition>,
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
    #[serde(default)]
    textures: IndexMap<String, VariableIdentifier>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ModelOrId {
    Id(Identifier),
    Model(Model),
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
    fn compile(
        &self,
        model: &mut Model,
        models: &HashMap<Identifier, Model>,
        variables: &HashMap<String, ModelOrId>,
    ) -> anyhow::Result<()> {
        let variable = variables
            .get(&self.model.clone().get_name())
            .with_context(|| format!("Failed to locate model variable: {}", self.model))?;
        let mut lookup_model = match variable {
            ModelOrId::Model(model) => model,
            ModelOrId::Id(id) => models
                .get(id)
                .with_context(|| format!("Failed to locate model: {}", id))?,
        }
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

        for (old, new) in &self.textures {
            lookup_model.update_texture(&VariableIdentifier::new(old.clone()), new.clone());
        }

        for element in &lookup_model.elements {
            model.elements.push(element.clone());
        }

        for (texture_name, texture_id) in &lookup_model.textures {
            model
                .textures
                .insert(texture_name.clone(), texture_id.clone());
        }

        if let Some(parent) = lookup_model.parent {
            model.parent = Some(parent.clone());
        }

        Ok(())
    }
}

impl ModelPreprocessed {
    pub fn compile(&self, models: &HashMap<Identifier, Model>) -> anyhow::Result<Model> {
        let mut model = Model::default();

        for part in &self.composition {
            part.compile(&mut model, models, &self.import)?;
        }

        Ok(model)
    }
}
