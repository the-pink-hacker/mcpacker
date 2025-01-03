use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use async_fs::File;
use futures_lite::AsyncWriteExt;
use indexmap::IndexMap;
use serde::Serialize;

use crate::{
    asset::{model::ModelGeneric, LoadableAsset},
    minecraft::asset::{
        atlas::Atlas,
        blockstate::Blockstate,
        model::Model,
        texture::TextureMeta,
        types::{
            identifier::{AssetType, Identifier},
            item::ItemModelDefinition,
        },
        Asset,
    },
};

use super::{
    dependency::DependencyGraph,
    modifier::{
        configurable::{
            ConfigurableModifier, ConfigurableModifierCollection, ConfigurableModifierIndex,
        },
        ModelModifiers,
    },
    PackCompiler,
};

#[derive(Debug, Default)]
pub struct AssetLibrary {
    pub models: HashMap<Identifier, ModelGeneric>,
    pub modifiers: HashMap<Identifier, ConfigurableModifierCollection>,
    pub modifier_index: ConfigurableModifierIndex,
    pub blockstates: HashMap<Identifier, Blockstate>,
    pub atlases: HashMap<Identifier, Atlas>,
    pub textures: HashMap<Identifier, PathBuf>,
    pub textures_meta: HashMap<Identifier, TextureMeta>,
    pub item_model_definitions: HashMap<Identifier, ItemModelDefinition>,
}

impl AssetLibrary {
    pub async fn load_asset(
        &mut self,
        asset_path: &Path,
        asset_path_absolute: &Path,
    ) -> anyhow::Result<()> {
        let (asset_type, id) = Identifier::from_path(asset_path)?;

        match asset_type {
            AssetType::Model => {
                Self::load_asset_generic(id, asset_path_absolute, &mut self.models).await
            }
            AssetType::Blockstate => {
                Self::load_asset_generic(id, asset_path_absolute, &mut self.blockstates).await
            }
            AssetType::Texture => {
                self.textures.insert(id, asset_path_absolute.to_owned());
                Ok(())
            }
            AssetType::Atlas => {
                Self::load_asset_generic(id, asset_path_absolute, &mut self.atlases).await
            }
            AssetType::TextureMeta => {
                Self::load_asset_generic(id, asset_path_absolute, &mut self.textures_meta).await
            }
            AssetType::Modifier => {
                Self::load_asset_generic(id, asset_path_absolute, &mut self.modifiers).await
            }
            AssetType::ModifierIndex => {
                Self::load_asset_single(asset_path_absolute, &mut self.modifier_index).await
            }
            AssetType::ItemModelDefinition => {
                Self::load_asset_generic(id, asset_path_absolute, &mut self.item_model_definitions)
                    .await
            }
            _ => Err(anyhow!("Asset type unsupported")),
        }
    }

    async fn load_asset_generic<A: LoadableAsset>(
        id: Identifier,
        path: impl AsRef<Path>,
        store: &mut HashMap<Identifier, A>,
    ) -> anyhow::Result<()> {
        let raw = async_fs::read_to_string(path).await?;
        let parsed = A::load_asset(raw)?;
        store.insert(id, parsed);

        Ok(())
    }

    async fn load_asset_single<A: LoadableAsset>(
        path: impl AsRef<Path>,
        store: &mut A,
    ) -> anyhow::Result<()> {
        let raw = async_fs::read_to_string(path).await?;
        let parsed = A::load_asset(raw)?;
        *store = parsed;

        Ok(())
    }

    pub fn compile(mut self) -> anyhow::Result<CompiledAssetLibrary> {
        let model_graph = DependencyGraph::from(&self.models).sort()?;

        let mut compiled_models = HashMap::with_capacity(self.models.len());
        let mut preprocessed_models = IndexMap::new();

        for model_id in model_graph {
            if let Some(model_generic) = self.models.remove(&model_id) {
                match model_generic {
                    ModelGeneric::Normal(model) => {
                        compiled_models.insert(model_id, *model);
                    }
                    ModelGeneric::Preprocessed(model) => {
                        preprocessed_models.insert(model_id, model);
                    }
                }
            }
        }

        for (preprocessed_model_id, preprocessed_model) in preprocessed_models
            .iter()
            .filter(|(_, model)| !model.is_virtual())
        {
            let compiled_model =
                preprocessed_model.compile(&compiled_models, &preprocessed_models)?;

            compiled_models.insert(preprocessed_model_id.clone(), compiled_model);
        }

        let mut modifiers = Vec::new();

        for modifier_collection_id in &self.modifier_index.order {
            let modifier_collection = self
                .modifiers
                .get(modifier_collection_id)
                .with_context(|| {
                    format!(
                        "Failed to lookup modifier collection: {}",
                        modifier_collection_id
                    )
                })?
                .modifiers
                .iter()
                .cloned()
                .map(ConfigurableModifier::into);
            modifiers.extend(modifier_collection);
        }

        Ok(CompiledAssetLibrary {
            models: compiled_models,
            modifiers,
            blockstates: self.blockstates,
            atlases: self.atlases,
            textures: self.textures,
            textures_meta: self.textures_meta,
            item_model_definitions: self.item_model_definitions,
        })
    }
}

impl PackCompiler<'_> {
    pub async fn populate_asset_library(&mut self) -> anyhow::Result<AssetLibrary> {
        let tracked_files = self.tracker.condence(&self.bundles)?;
        let mut library = AssetLibrary::default();

        for file in tracked_files {
            let asset_path = file
                .strip_prefix(&self.bundles_path)?
                .iter()
                .skip(1)
                .collect::<PathBuf>();

            if let Err(e) = library.load_asset(&asset_path, &file).await {
                println!("[WARNING] Parse error at \"{}\":\n{}", file.display(), e);
            }
        }

        Ok(library)
    }
}

pub struct CompiledAssetLibrary {
    pub models: HashMap<Identifier, Model>,
    pub modifiers: ModelModifiers,
    pub blockstates: HashMap<Identifier, Blockstate>,
    pub atlases: HashMap<Identifier, Atlas>,
    pub textures: HashMap<Identifier, PathBuf>,
    pub textures_meta: HashMap<Identifier, TextureMeta>,
    pub item_model_definitions: HashMap<Identifier, ItemModelDefinition>,
}

impl CompiledAssetLibrary {
    pub async fn write_contents(&self, compiler: &PackCompiler<'_>) -> anyhow::Result<()> {
        Self::write_asset_collection(compiler, &self.models).await?;
        Self::write_asset_collection(compiler, &self.blockstates).await?;
        Self::write_asset_collection(compiler, &self.atlases).await?;
        Self::write_asset_collection(compiler, &self.textures_meta).await?;
        Self::write_asset_collection(compiler, &self.item_model_definitions).await?;

        for (id, texture) in &self.textures {
            Self::copy_asset(compiler, id, texture, &AssetType::Texture).await?;
        }

        Ok(())
    }

    async fn write_asset_collection<T: Asset + Serialize>(
        compiler: &PackCompiler<'_>,
        collection: &HashMap<Identifier, T>,
    ) -> anyhow::Result<()> {
        for (id, asset) in collection {
            Self::write_asset(compiler, id, asset).await?;
        }

        Ok(())
    }

    async fn write_asset<T: Asset + Serialize>(
        compiler: &PackCompiler<'_>,
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
        async_fs::create_dir_all(output_folder).await?;

        let output = compiler.profile.json_type.to_string(&asset)?;

        let mut output_file = File::create(output_file_path).await?;
        output_file.write_all(output.as_bytes()).await?;
        output_file.flush().await?;

        Ok(())
    }

    async fn copy_asset<P: AsRef<Path>>(
        compiler: &PackCompiler<'_>,
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
        async_fs::create_dir_all(output_folder).await?;

        async_fs::copy(asset, output_file_path).await?;

        Ok(())
    }
}
