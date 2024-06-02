use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use crate::minecraft::asset::{
    atlas::Atlas,
    blockstate::Blockstate,
    model::Model,
    types::identifier::{AssetType, Identifier},
};

use super::PackCompiler;

#[derive(Debug, Default)]
pub struct AssetLibrary {
    models: HashMap<Identifier, Model>,
    blockstates: HashMap<Identifier, Blockstate>,
    atlases: HashMap<Identifier, Atlas>,
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
            AssetType::Atlas => self.load_atlas(id, asset_path_absolute),
        }
    }

    pub fn load_model(&mut self, id: Identifier, path: &Path) -> anyhow::Result<()> {
        let raw = std::fs::read_to_string(path)?;
        let parsed = serde_json::from_str(&raw)?;

        self.models.insert(id, parsed);

        Ok(())
    }

    pub fn load_blockstate(&mut self, id: Identifier, path: &Path) -> anyhow::Result<()> {
        let raw = std::fs::read_to_string(path)?;
        let parsed = serde_json::from_str(&raw)?;

        self.blockstates.insert(id, parsed);

        Ok(())
    }

    pub fn load_atlas(&mut self, id: Identifier, path: &Path) -> anyhow::Result<()> {
        let raw = std::fs::read_to_string(path)?;
        let parsed = serde_json::from_str(&raw)?;

        self.atlases.insert(id, parsed);

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

    pub fn write_asset_library(&self) -> anyhow::Result<()> {
        for (id, model) in &self.library.models {
            self.write_asset(id, model, &AssetType::Model)?;
        }

        for (id, blockstate) in &self.library.blockstates {
            self.write_asset(id, blockstate, &AssetType::Blockstate)?;
        }

        for (id, atlas) in &self.library.atlases {
            self.write_asset(id, atlas, &AssetType::Atlas)?;
        }

        Ok(())
    }

    fn write_asset(
        &self,
        id: &Identifier,
        asset: impl serde::Serialize,
        asset_type: &AssetType,
    ) -> anyhow::Result<()> {
        let output_file_path = id.to_path(&self.asset_path, asset_type);

        let mut output_path = output_file_path.clone();
        output_path.pop();
        std::fs::create_dir_all(&output_path)?;

        let output = self.profile.json_type.to_string(&asset)?;

        let mut output_file = File::create(output_file_path)?;
        output_file.write_all(output.as_bytes())?;

        Ok(())
    }
}
