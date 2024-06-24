use std::collections::{HashMap, HashSet};

use anyhow::bail;
use indexmap::IndexSet;
use topological_sort::TopologicalSort;

use crate::{
    asset::model::{ModelGeneric, ModelOrId, ModelPreprocessed},
    minecraft::asset::{model::Model, types::identifier::Identifier},
};

#[derive(Debug, Default)]
pub struct DependencyGraph<'a> {
    id_table: IndexSet<&'a Identifier>,
    // IDs are converted to usizes for preformance.
    // Reduces amount of hashing.
    // Haven't tested this since refactoring.
    graph: TopologicalSort<usize>,
}

impl<'a> DependencyGraph<'a> {
    pub fn sort(mut self) -> anyhow::Result<Vec<Identifier>> {
        let mut output = Vec::with_capacity(self.graph.len());

        while !self.graph.is_empty() {
            let dependency_order = self
                .graph
                .pop_all()
                .iter()
                .map(|i| self.index_to_id(*i))
                .filter_map(Option::from)
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

    fn id_to_index(&mut self, id: &'a Identifier) -> usize {
        let (index, _) = self.id_table.insert_full(id);
        index
    }

    fn index_to_id(&self, index: usize) -> Option<&'a Identifier> {
        self.id_table.get_index(index).copied()
    }

    fn add_model(&mut self, id: &'a Identifier, model: &'a Model) {
        let to = self.id_to_index(id);

        if let Some(parent) = &model.parent {
            let from = self.id_to_index(parent);
            self.graph.add_dependency(from, to);
        }
    }

    fn add_model_preprocessed(&mut self, id: &'a Identifier, model: &'a ModelPreprocessed) {
        let imports = model
            .import
            .values()
            .map(|m| match m {
                ModelOrId::Id(id) => Some(id),
                ModelOrId::Model(model) => model.parent.as_ref(),
            })
            .filter_map(Option::from)
            .collect::<HashSet<_>>();

        let to = self.id_to_index(id);

        for import in imports {
            let from = self.id_to_index(import);
            self.graph.add_dependency(from, to);
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
