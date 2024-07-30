use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use rand::SeedableRng;
use rand_pcg::Mcg128Xsl64;

use crate::{
    config::{CollectionConfig, PackMetaConfig, ProfileConfig},
    sanitize::PathSanitizer,
};

use self::tracking::AssetTracker;

pub mod compiler;
pub mod dependency;
pub mod deploy;
pub mod library;
pub mod modifier;
pub mod tracking;

pub struct PackCompiler<'a> {
    project_sanitizer: &'a PathSanitizer,
    compile_path: PathBuf,
    bundles_path: PathBuf,
    resourcepack_path: PathBuf,
    pack: PackMetaConfig,
    profile: Arc<ProfileConfig>,
    bundles: Vec<PathBuf>,
    tracker: Arc<AssetTracker>,
    minecraft_versions: Vec<String>,
    pub rand: Mcg128Xsl64,
}

impl<'a> PackCompiler<'a> {
    pub fn new(
        project_sanitizer: &'a PathSanitizer,
        minecraft_path: PathBuf,
        pack: PackMetaConfig,
        profile: Arc<ProfileConfig>,
        build: CollectionConfig,
        tracker: Arc<AssetTracker>,
    ) -> anyhow::Result<Self> {
        let name = pack
            .name
            .clone()
            .filter(|t| !t.is_empty())
            .with_context(|| "pack name is empty")?
            .to_string();

        let compile_path = project_sanitizer.join(PathBuf::from("build").join(&name))?;
        let source_path = project_sanitizer.restricted_path.join("src");

        let mut compiler = Self {
            project_sanitizer,
            rand: Mcg128Xsl64::seed_from_u64(pack.seed.unwrap_or_default()),
            pack,
            profile,
            bundles_path: source_path.join("bundles"),
            resourcepack_path: minecraft_path.join("resourcepacks").join(name),
            compile_path,
            tracker,
            bundles: Vec::with_capacity(build.bundles.len()),
            minecraft_versions: build.minecraft_versions,
        };

        for bundle in build.bundles {
            compiler.bundles.push(compiler.get_bundle_path(bundle)?);
        }

        Ok(compiler)
    }
}
