use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

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
    pub fn load_asset(
        &mut self,
        asset_path: &Path,
        asset_path_absolute: &Path,
    ) -> anyhow::Result<()> {
        let (asset_type, id) = Identifier::from_path(asset_path)?;

        match asset_type {
            AssetType::Model => self.load_model(id, asset_path_absolute),
            AssetType::Blockstate => self.load_blockstate(id, asset_path_absolute),
            AssetType::Texture => Ok(()),
            AssetType::Atlas => Ok(()),
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
            let absolute_path = self.bundles_path.join(&file);
            let asset_path = file.iter().skip(1).collect::<PathBuf>();

            if let Err(e) = self.library.load_asset(&asset_path, &absolute_path) {
                println!(
                    "[WARNING] Parse error at \"{}\":\n{}",
                    absolute_path.display(),
                    e
                );
            }
        }

        Ok(())
    }
}
