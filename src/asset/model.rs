use std::collections::HashMap;

use anyhow::{bail, Context};
use indexmap::IndexMap;
use serde::Deserialize;

use crate::minecraft::asset::{
    model::{flip::Flip, rotate::Rotatable, translate::Translate, Axis, CullDirection, Model},
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
    Normal(Box<Model>),
}

impl LoadableAsset for ModelGeneric {
    fn load_asset<R: AsRef<str>>(raw: R) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(raw.as_ref())?)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelPreprocessed {
    #[serde(default)]
    pub import: HashMap<String, ModelOrId>,
    pub composition: ModelComposition,
    #[serde(default, rename = "virtual")]
    is_virtual: bool,
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
pub struct ModelPart {
    model: VariableIdentifier,
    #[serde(default)]
    optional: bool,
    #[serde(default)]
    transformations: Vec<Transformation>,
    cullface: Option<CullDirection>,
    #[serde(default)]
    textures: IndexMap<String, VariableIdentifier>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ModelOrId {
    Id(Identifier),
    Model(Box<Model>),
}

#[derive(Debug, Deserialize, Clone)]
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
    Flip {
        axis: Axis,
    },
}

impl ModelPreprocessed {
    pub fn compile(
        &self,
        raw_models: &HashMap<Identifier, Model>,
        prepocessed_models: &IndexMap<Identifier, ModelPreprocessed>,
    ) -> anyhow::Result<Model> {
        let mut builder = ModelBuilder::new(raw_models, prepocessed_models, self.import.clone());

        builder.add_compositon(&self.composition)?;

        Ok(builder.build())
    }
}

struct ModelBuilder<'a> {
    raw_models: &'a HashMap<Identifier, Model>,
    prepocessed_models: &'a IndexMap<Identifier, ModelPreprocessed>,
    output_model: Model,
    import_table: HashMap<String, ModelOrId>,
}

impl<'a> ModelBuilder<'a> {
    fn new(
        raw_models: &'a HashMap<Identifier, Model>,
        prepocessed_models: &'a IndexMap<Identifier, ModelPreprocessed>,
        import_table: HashMap<String, ModelOrId>,
    ) -> Self {
        Self {
            raw_models,
            prepocessed_models,
            import_table,
            output_model: Model::default(),
        }
    }

    fn add_compositon(&mut self, composition: &ModelComposition) -> anyhow::Result<()> {
        let parts = match composition {
            ModelComposition::Parts(parts) => parts,
            ModelComposition::Template(template_id) => self.resolve_template(template_id)?,
        };

        self.add_parts(parts)?;

        Ok(())
    }

    fn add_parts<'b, P: IntoIterator<Item = &'b ModelPart>>(
        &mut self,
        parts: P,
    ) -> anyhow::Result<()> {
        for part in parts {
            self.add_part(part)?;
        }

        Ok(())
    }

    fn add_part(&mut self, part: &ModelPart) -> anyhow::Result<()> {
        let model_reference = {
            match self.evaluate_model_variable(&part.model) {
                Some(model_reference) => model_reference,
                None => {
                    if part.optional {
                        return Ok(());
                    } else {
                        bail!("Failed to evaluate model variable: {}", part.model);
                    }
                }
            }
        };

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
                Transformation::Flip { axis } => lookup_model.flip(axis),
            }
        }

        for (old_texture_id, new_texture_id) in part.textures.clone() {
            lookup_model.update_texture(&VariableIdentifier::new(old_texture_id), new_texture_id);
        }

        self.output_model
            .elements
            .append(&mut lookup_model.elements);

        for (texture_name, texture_id) in lookup_model.textures.clone() {
            self.output_model.textures.insert(texture_name, texture_id);
        }

        if let Some(parent) = lookup_model.parent {
            self.output_model.parent = Some(parent);
        }

        if let Some(display) = lookup_model.display {
            self.output_model.display.replace(display);
        }

        Ok(())
    }

    fn resolve_template(&mut self, template_id: &Identifier) -> anyhow::Result<&'a Vec<ModelPart>> {
        let template = self.lookup_preprocessed_model(template_id)?;

        let mut new_import_table = template.import.clone();
        new_import_table.extend(self.import_table.clone());
        self.import_table = new_import_table;

        match &template.composition {
            ModelComposition::Template(id) => self.resolve_template(id),
            ModelComposition::Parts(parts) => Ok(parts),
        }
    }

    fn lookup_model(&self, model_id: &Identifier) -> anyhow::Result<&'a Model> {
        self.raw_models
            .get(model_id)
            .with_context(|| format!("Failed to lookup model: {}", model_id))
    }

    fn lookup_preprocessed_model(
        &self,
        model_id: &Identifier,
    ) -> anyhow::Result<&'a ModelPreprocessed> {
        self.prepocessed_models
            .get(model_id)
            .with_context(|| format!("Failed to lookup prepocessed model: {}", model_id))
    }

    fn evaluate_model_variable(&'a self, variable: &VariableIdentifier) -> Option<&'a ModelOrId> {
        self.import_table.get(variable.get_name())
    }

    fn build(self) -> Model {
        self.output_model
    }
}
