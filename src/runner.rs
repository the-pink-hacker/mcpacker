use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Context;

use crate::{
    compile::{deploy::DeployAPIContext, tracking::AssetTracker, PackCompiler},
    config::PackConfig,
    sanitize::PathSanitizer,
};

pub struct Runner {
    project_sanitizer: PathSanitizer,
    config: PathBuf,
    minecraft_path: PathBuf,
    builds: Vec<String>,
    profile: String,
    api_context: Arc<Mutex<DeployAPIContext>>,
}

impl Runner {
    pub fn new(
        config: PathBuf,
        minecraft_path: PathBuf,
        builds: Vec<String>,
        profile: String,
    ) -> anyhow::Result<Self> {
        let project_sanitizer = config
            .parent()
            .with_context(|| format!("Failed to get project path at: {}", config.display()))?
            .try_into()?;
        Ok(Self {
            project_sanitizer,
            config,
            minecraft_path,
            builds,
            profile,
            api_context: Default::default(),
        })
    }

    pub async fn start_standard(self) -> anyhow::Result<()> {
        let compilers = self.create_compilers()?;

        for mut compiler in compilers {
            compiler.run().await;
            compiler.deploy(self.api_context.as_ref()).await?;
        }

        Ok(())
    }

    fn create_compilers(&self) -> anyhow::Result<Vec<PackCompiler>> {
        let config_raw = std::fs::read_to_string(&self.config).context("Config read error.")?;
        let config = toml::from_str::<PackConfig>(&config_raw).context("Config parse error.")?;

        let mut asset_tracker = AssetTracker::default();

        asset_tracker.search_bundle_files(
            &self
                .project_sanitizer
                .restricted_path
                .join("src")
                .join("bundles"),
        )?;

        let asset_tracker = Arc::from(asset_tracker);

        let mut compilers = Vec::with_capacity(self.builds.len());
        let profile = config.get_profile(&self.profile)?;

        for build_name in &self.builds {
            let build = config.get_build(build_name)?.clone();
            let pack = config.condence_packs(&build.pack, &profile.pack);
            let compiler = PackCompiler::new(
                &self.project_sanitizer,
                self.minecraft_path.clone(),
                pack,
                profile.clone(),
                build,
                asset_tracker.clone(),
            )?;
            compilers.push(compiler);
        }

        Ok(compilers)
    }
}
