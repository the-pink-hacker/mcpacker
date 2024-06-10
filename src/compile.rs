use std::{path::PathBuf, sync::Arc};

use anyhow::Context;

use crate::{
    config::{CollectionConfig, PackMetaConfig, ProfileConfig},
    minecraft::asset::types::text::RawText,
};

use self::{library::AssetLibrary, tracking::AssetTracker};

pub mod compiler;
pub mod library;
pub mod redirect;
pub mod tracking;

pub struct PackCompiler {
    compile_path: PathBuf,
    bundles_path: PathBuf,
    project_path: PathBuf,
    resourcepack_path: PathBuf,
    pack: PackMetaConfig,
    profile: Arc<ProfileConfig>,
    bundles: Vec<PathBuf>,
    redirects: Vec<PathBuf>,
    library: AssetLibrary,
    tracker: Arc<AssetTracker>,
}

impl PackCompiler {
    pub fn new(
        project_path: PathBuf,
        minecraft_path: PathBuf,
        pack: PackMetaConfig,
        profile: Arc<ProfileConfig>,
        build: CollectionConfig,
        tracker: Arc<AssetTracker>,
    ) -> anyhow::Result<Self> {
        let name = pack
            .name
            .clone()
            .filter(RawText::is_empty)
            .with_context(|| "pack name is empty")?
            .to_string();

        let compile_path = project_path.join("build").join(&name);

        Ok(Self {
            pack,
            profile,
            bundles_path: project_path
                .join("src")
                .canonicalize()
                .expect("Failed to get absolute bundle path."),
            resourcepack_path: minecraft_path.join("resourcepacks").join(name),
            compile_path,
            library: Default::default(),
            tracker,
            redirects: build.redirects,
            bundles: build.bundles,
            project_path,
        })
    }
}
