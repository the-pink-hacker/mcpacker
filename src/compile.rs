use std::{path::PathBuf, sync::Arc};

use anyhow::Context;

use crate::{
    config::{CollectionConfig, PackMetaConfig, ProfileConfig},
    sanitize::PathSanitizer,
};

use self::{library::AssetLibrary, tracking::AssetTracker};

pub mod compiler;
pub mod library;
pub mod redirect;
pub mod tracking;

pub struct PackCompiler<'a> {
    project_sanitizer: &'a PathSanitizer,
    compile_path: PathBuf,
    bundles_path: PathBuf,
    redirects_path: PathBuf,
    resourcepack_path: PathBuf,
    pack: PackMetaConfig,
    profile: Arc<ProfileConfig>,
    bundles: Vec<PathBuf>,
    redirects: Vec<PathBuf>,
    library: AssetLibrary,
    tracker: Arc<AssetTracker>,
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
            pack,
            profile,
            bundles_path: source_path.join("bundles"),
            redirects_path: source_path.join("redirects"),
            resourcepack_path: minecraft_path.join("resourcepacks").join(name),
            compile_path,
            library: Default::default(),
            tracker,
            redirects: Vec::with_capacity(build.redirects.len()),
            bundles: Vec::with_capacity(build.bundles.len()),
        };

        for bundle in build.bundles {
            compiler.bundles.push(compiler.get_bundle_path(bundle)?);
        }

        for redirect in build.redirects {
            compiler
                .redirects
                .push(compiler.get_redirect_path(redirect)?);
        }

        Ok(compiler)
    }
}
