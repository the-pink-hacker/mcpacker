use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use serde::Serialize;

use crate::{
    asset::{model::ModelGeneric, LoadableAsset},
    minecraft::asset::{
        atlas::Atlas,
        blockstate::Blockstate,
        model::Model,
        texture::TextureMeta,
        types::identifier::{AssetType, Identifier},
        Asset,
    },
};

use super::{dependency::DependencyGraph, PackCompiler};

#[derive(Debug, Default)]
pub struct AssetLibrary {
    pub models: HashMap<Identifier, ModelGeneric>,
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
            AssetType::Model => Self::load_asset_generic(id, asset_path_absolute, &mut self.models),
            AssetType::Blockstate => {
                Self::load_asset_generic(id, asset_path_absolute, &mut self.blockstates)
            }
            AssetType::Texture => {
                self.textures.insert(id, asset_path_absolute.to_owned());
                Ok(())
            }
            AssetType::Atlas => {
                Self::load_asset_generic(id, asset_path_absolute, &mut self.atlases)
            }
            AssetType::TextureMeta => {
                Self::load_asset_generic(id, asset_path_absolute, &mut self.textures_meta)
            }
            _ => Err(anyhow!("Asset type unsupported")),
        }
    }

    fn load_asset_generic<A: LoadableAsset, P: AsRef<Path>>(
        id: Identifier,
        path: P,
        store: &mut HashMap<Identifier, A>,
    ) -> anyhow::Result<()> {
        let raw = std::fs::read_to_string(path)?;
        let parsed = A::load_asset(raw)?;
        store.insert(id, parsed);

        Ok(())
    }

    pub fn compile(mut self) -> anyhow::Result<CompiledAssetLibrary> {
        let model_graph = DependencyGraph::from(&self.models).sort()?;

        let mut compiled_models = HashMap::with_capacity(self.models.len());

        for model_id in model_graph {
            if let Some(model_generic) = self.models.remove(&model_id) {
                let compiled_model = match model_generic {
                    ModelGeneric::Normal(model) => model,
                    ModelGeneric::Preprocessed(model) => model.compile(&compiled_models)?,
                };

                compiled_models.insert(model_id, compiled_model);
            }
        }

        Ok(CompiledAssetLibrary {
            models: compiled_models,
            blockstates: self.blockstates,
            atlases: self.atlases,
            textures: self.textures,
            textures_meta: self.textures_meta,
        })
    }
}

impl<'a> PackCompiler<'a> {
    pub fn populate_asset_library(&mut self) -> anyhow::Result<AssetLibrary> {
        let tracked_files = self.tracker.condence(&self.bundles)?;
        let mut library = AssetLibrary::default();

        for file in tracked_files {
            let asset_path = file
                .strip_prefix(&self.bundles_path)?
                .iter()
                .skip(1)
                .collect::<PathBuf>();

            if let Err(e) = library.load_asset(&asset_path, &file) {
                println!("[WARNING] Parse error at \"{}\":\n{}", file.display(), e);
            }
        }

        Ok(library)
    }
}

#[derive(Debug)]
pub struct CompiledAssetLibrary {
    pub models: HashMap<Identifier, Model>,
    pub blockstates: HashMap<Identifier, Blockstate>,
    pub atlases: HashMap<Identifier, Atlas>,
    pub textures: HashMap<Identifier, PathBuf>,
    pub textures_meta: HashMap<Identifier, TextureMeta>,
}

impl CompiledAssetLibrary {
    pub fn write_contents(&self, compiler: &PackCompiler) -> anyhow::Result<()> {
        Self::write_asset_collection(compiler, &self.models)?;
        Self::write_asset_collection(compiler, &self.blockstates)?;
        Self::write_asset_collection(compiler, &self.atlases)?;
        Self::write_asset_collection(compiler, &self.textures_meta)?;

        for (id, texture) in &self.textures {
            Self::copy_asset(compiler, id, texture, &AssetType::Texture)?;
        }

        Ok(())
    }

    fn write_asset_collection<T: Asset + Serialize>(
        compiler: &PackCompiler,
        collection: &HashMap<Identifier, T>,
    ) -> anyhow::Result<()> {
        for (id, asset) in collection {
            Self::write_asset(compiler, id, asset)?;
        }

        Ok(())
    }

    fn write_asset<T: Asset + Serialize>(
        compiler: &PackCompiler,
        id: &Identifier,
        asset: &T,
    ) -> anyhow::Result<()> {
        if asset.is_virtual() {
            return Ok(());
        }

        let output_file_path = id.to_path(&compiler.compile_path.join("assets"), &T::get_type());

        let output_folder = output_file_path.parent().with_context(|| {
            format!(
                "Failed to get asset folder path from: {}",
                output_file_path.display()
            )
        })?;
        std::fs::create_dir_all(output_folder)?;

        let output = compiler.profile.json_type.to_string(&asset)?;

        let mut output_file = File::create(output_file_path)?;
        output_file.write_all(output.as_bytes())?;

        Ok(())
    }

    fn copy_asset<P: AsRef<Path>>(
        compiler: &PackCompiler,
        id: &Identifier,
        asset: P,
        asset_type: &AssetType,
    ) -> anyhow::Result<()> {
        let output_file_path = id.to_path(&compiler.compile_path.join("assets"), asset_type);

        let output_folder = output_file_path.parent().with_context(|| {
            format!(
                "Failed to get asset folder path from: {}",
                output_file_path.display()
            )
        })?;
        std::fs::create_dir_all(output_folder)?;

        std::fs::copy(asset, output_file_path)?;

        Ok(())
    }
}
