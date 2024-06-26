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
    pub composition: ModelComposition,
    #[serde(default)]
    is_virtual: bool,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ModelComposition {
    Template(Identifier),
    Parts(Vec<ModelPart>),
}

impl Asset for ModelPreprocessed {
    fn get_type() -> AssetType {
        AssetType::Model
    }

    fn is_virtual(&self) -> bool {
        self.is_virtual
    }
}

#[derive(Debug, Deserialize)]
pub struct ModelPart {
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

impl ModelPreprocessed {
    pub fn compile(
        &self,
        raw_models: &HashMap<Identifier, Model>,
        prepocessed_models: &IndexMap<Identifier, ModelPreprocessed>,
    ) -> anyhow::Result<Model> {
        let mut builder = ModelBuilder::new(raw_models, prepocessed_models, &self.import);

        match &self.composition {
            ModelComposition::Parts(parts) => {
                for part in parts {
                    builder.add_part(part)?;
                }
            }
            ModelComposition::Template(template_id) => todo!(),
        }

        Ok(builder.build())
    }
}

struct ModelBuilder<'a> {
    raw_models: &'a HashMap<Identifier, Model>,
    prepocessed_models: &'a IndexMap<Identifier, ModelPreprocessed>,
    model: Model,
    import_table: &'a HashMap<String, ModelOrId>,
}

impl<'a> ModelBuilder<'a> {
    fn new(
        raw_models: &'a HashMap<Identifier, Model>,
        prepocessed_models: &'a IndexMap<Identifier, ModelPreprocessed>,
        import_table: &'a HashMap<String, ModelOrId>,
    ) -> Self {
        Self {
            raw_models,
            prepocessed_models,
            model: Model::default(),
            import_table,
        }
    }

    fn add_part(&mut self, part: &ModelPart) -> anyhow::Result<()> {
        let model_reference = self
            .evaluate_model_variable(&part.model)
            .with_context(|| format!("Failed to locate model variable: {}", part.model))?;

        let mut lookup_model = match model_reference {
            ModelOrId::Model(model) => model,
            ModelOrId::Id(id) => self.lookup_model(id)?,
        }
        .clone();

        if let Some(cullface) = &part.cullface {
            lookup_model.set_cullface(cullface);
        }

        for transform in &part.transformations {
            match transform {
                Transformation::Rotate { x, y } => {
                    lookup_model.rotate_x(x);
                    lookup_model.rotate_y(y);
                }
                Transformation::Translate { amount } => lookup_model.translate(amount),
            }
        }

        for (old_texture_id, new_texture_id) in part.textures.clone() {
            lookup_model.update_texture(&VariableIdentifier::new(old_texture_id), new_texture_id);
        }

        for element in lookup_model.elements.clone() {
            self.model.elements.push(element);
        }

        for (texture_name, texture_id) in lookup_model.textures.clone() {
            self.model.textures.insert(texture_name, texture_id);
        }

        if let Some(parent) = lookup_model.parent {
            self.model.parent = Some(parent.clone());
        }

        Ok(())
    }

    fn lookup_model(&mut self, model_id: &Identifier) -> anyhow::Result<&'a Model> {
        self.raw_models
            .get(model_id)
            .with_context(|| format!("Failed to lookup model: {}", model_id))
    }

    fn evaluate_model_variable(&self, variable: &VariableIdentifier) -> Option<&'a ModelOrId> {
        self.import_table.get(variable.get_name())
    }

    fn build(self) -> Model {
        self.model
    }
}
