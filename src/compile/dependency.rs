use std::collections::{HashMap, HashSet};

use anyhow::bail;
use topological_sort::TopologicalSort;

use crate::{
    asset::model::{ModelComposition, ModelGeneric, ModelOrId, ModelPreprocessed},
    minecraft::asset::{model::Model, types::identifier::Identifier},
};

#[derive(Debug, Default)]
pub struct DependencyGraph<'a> {
    graph: TopologicalSort<&'a Identifier>,
}

impl<'a> DependencyGraph<'a> {
    pub fn sort(mut self) -> anyhow::Result<Vec<Identifier>> {
        let mut output = Vec::with_capacity(self.graph.len());

        while !self.graph.is_empty() {
            let dependency_order = self
                .graph
                .pop_all()
                .iter()
                .copied()
                .map(Identifier::clone)
                .collect::<Vec<_>>();

            if !self.graph.is_empty() && dependency_order.is_empty() {
                bail!("Dependency graph is cyclic; circular dependency detected.");
            } else {
                output.extend(dependency_order);
            }
        }

        Ok(output)
    }

    fn add_model(&mut self, id: &'a Identifier, model: &'a Model) {
        if let Some(parent) = &model.parent {
            self.graph.add_dependency(parent, id);
        } else {
            self.graph.insert(id);
        }
    }

    fn add_model_preprocessed(&mut self, model_id: &'a Identifier, model: &'a ModelPreprocessed) {
        let imports = model
            .import
            .values()
            .map(|m| match m {
                ModelOrId::Id(id) => Some(id),
                ModelOrId::Model(model) => model.parent.as_ref(),
            })
            .filter_map(Option::from)
            .collect::<HashSet<&Identifier>>();

        if let ModelComposition::Template(template_id) = &model.composition {
            self.graph.add_dependency(template_id, model_id);
        }

        if imports.is_empty() {
            // There are no references to this model in the graph.
            // Must add to graph manually.
            self.graph.insert(model_id);
        } else {
            for import in imports {
                self.graph.add_dependency(import, model_id);
            }
        }
    }
}

impl<'a> From<&'a HashMap<Identifier, ModelGeneric>> for DependencyGraph<'a> {
    fn from(value: &'a HashMap<Identifier, ModelGeneric>) -> Self {
        let mut output = Self::default();

        for (id, model) in value {
            match model {
                ModelGeneric::Preprocessed(model) => output.add_model_preprocessed(id, model),
                ModelGeneric::Normal(model) => output.add_model(id, model),
            }
        }

        output
    }
}
