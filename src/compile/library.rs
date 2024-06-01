use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Context;

use crate::minecraft::asset::{
    blockstate::Blockstate,
    model::Model,
    types::identifier::{AssetType, Identifier},
};

use super::PackCompiler;

#[derive(Debug, Default)]
pub struct AssetLibrary {
    models: HashMap<Identifier, Model>,
    blockstates: HashMap<Identifier, Blockstate>,
}

impl AssetLibrary {
    pub fn load_asset(&mut self, asset_path: &Path, bundle_path: &Path) -> anyhow::Result<()> {
        let (asset_type, id) = Identifier::from_path(asset_path)?;
        let path = bundle_path.join(asset_path);
        println!("{}", path.display());

        match asset_type {
            AssetType::Model => self.load_model(id, &path),
            AssetType::Blockstate => self.load_blockstate(id, &path),
        }
    }

    pub fn load_model(&mut self, id: Identifier, path: &Path) -> anyhow::Result<()> {
        let raw_model = std::fs::read_to_string(path)?;
        let model = serde_json::from_str(&raw_model)?;

        self.models.insert(id, model);

        Ok(())
    }

    pub fn load_blockstate(&mut self, id: Identifier, path: &Path) -> anyhow::Result<()> {
        let raw_model = std::fs::read_to_string(path)?;
        let model = serde_json::from_str(&raw_model)?;

        self.blockstates.insert(id, model);

        Ok(())
    }
}

impl PackCompiler {
    pub fn populate_asset_library(&mut self) -> anyhow::Result<()> {
        let bundle_order = &self.build.bundles;
        let tracked_files = self.tracker.condence(bundle_order)?;

        for file in tracked_files {
            let mut file_parts = file.iter();
            let bundle_path = self.bundles_path.join(file_parts.next().with_context(|| {
                format!("Failed to parse bundle name from path: {}", file.display())
            })?);
            let asset_path = file_parts.collect::<PathBuf>();

            self.library.load_asset(&asset_path, &bundle_path)?;
        }

        Ok(())
    }
}
