use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use serde::Serialize;

use crate::{
    asset::model::ModelPreprocessed,
    minecraft::asset::{
        atlas::Atlas,
        blockstate::Blockstate,
        model::Model,
        texture::TextureMeta,
        types::identifier::{AssetType, Identifier},
        Asset,
    },
};

use super::PackCompiler;

#[derive(Debug, Default)]
pub struct AssetLibrary {
    pub models: HashMap<Identifier, Model>,
    pub models_preprocessed: HashMap<Identifier, ModelPreprocessed>,
    pub blockstates: HashMap<Identifier, Blockstate>,
    pub atlases: HashMap<Identifier, Atlas>,
    pub textures: HashMap<Identifier, PathBuf>,
    pub textures_meta: HashMap<Identifier, TextureMeta>,
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
            AssetType::ModelPreprocessed => self.load_model_preprocessed(id, asset_path_absolute),
            AssetType::Blockstate => self.load_blockstate(id, asset_path_absolute),
            AssetType::Texture => {
                self.textures.insert(id, asset_path_absolute.to_owned());
                Ok(())
            }
            AssetType::Atlas => self.load_atlas(id, asset_path_absolute),
            AssetType::TextureMeta => self.load_texture_meta(id, asset_path_absolute),
            _ => Err(anyhow!("Asset type unsupported")),
        }
    }

    pub fn load_model(&mut self, id: Identifier, path: &Path) -> anyhow::Result<()> {
        let raw = std::fs::read_to_string(path)?;
        let parsed = serde_json::from_str(&raw)?;

        self.models.insert(id, parsed);

        Ok(())
    }

    pub fn load_model_preprocessed(&mut self, id: Identifier, path: &Path) -> anyhow::Result<()> {
        let raw = std::fs::read_to_string(path)?;
        let parsed = serde_json::from_str(&raw)?;

        self.models_preprocessed.insert(id, parsed);

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

    pub fn load_texture_meta(&mut self, id: Identifier, path: &Path) -> anyhow::Result<()> {
        let raw = std::fs::read_to_string(path)?;
        let parsed = serde_json::from_str(&raw)?;

        self.textures_meta.insert(id, parsed);

        Ok(())
    }
}

impl<'a> PackCompiler<'a> {
    pub fn populate_asset_library(&mut self) -> anyhow::Result<()> {
        let tracked_files = self.tracker.condence(&self.bundles)?;

        for file in tracked_files {
            let asset_path = file
                .strip_prefix(&self.bundles_path)?
                .iter()
                .skip(1)
                .collect::<PathBuf>();

            if let Err(e) = self.library.load_asset(&asset_path, &file) {
                println!("[WARNING] Parse error at \"{}\":\n{}", file.display(), e);
            }
        }

        Ok(())
    }

    pub fn write_asset_library(&self) -> anyhow::Result<()> {
        for (id, model) in &self.library.models {
            self.write_asset(id, model)?;
        }

        for (id, model) in &self.library.models_preprocessed {
            self.write_asset(id, &model.compile(&self.library.models)?)?;
        }

        for (id, blockstate) in &self.library.blockstates {
            self.write_asset(id, blockstate)?;
        }

        for (id, atlas) in &self.library.atlases {
            self.write_asset(id, atlas)?;
        }

        for (id, texture) in &self.library.textures {
            self.copy_asset(id, texture, &AssetType::Texture)?;
        }

        for (id, texture_meta) in &self.library.textures_meta {
            self.write_asset(id, texture_meta)?;
        }

        Ok(())
    }

    fn write_asset<T: Asset + Serialize>(&self, id: &Identifier, asset: &T) -> anyhow::Result<()> {
        if id.is_virtual() {
            return Ok(());
        }

        let output_file_path = id.to_path(&self.compile_path.join("assets"), &T::get_type());

        let mut output_path = output_file_path.clone();
        output_path.pop();
        std::fs::create_dir_all(&output_path)?;

        let output = self.profile.json_type.to_string(&asset)?;

        let mut output_file = File::create(output_file_path)?;
        output_file.write_all(output.as_bytes())?;

        Ok(())
    }

    fn copy_asset(
        &self,
        id: &Identifier,
        asset: &Path,
        asset_type: &AssetType,
    ) -> anyhow::Result<()> {
        let output_file_path = id.to_path(&self.compile_path.join("assets"), asset_type);

        let mut output_path = output_file_path.clone();
        output_path.pop();
        std::fs::create_dir_all(&output_path)?;

        std::fs::copy(asset, output_file_path)?;

        Ok(())
    }
}
