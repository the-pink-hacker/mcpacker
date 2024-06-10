use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use notify::{INotifyWatcher, RecursiveMode, Watcher};
use rayon::prelude::*;

use crate::{
    compile::{tracking::AssetTracker, PackCompiler},
    config::PackConfig,
};

pub struct Runner {
    config: PathBuf,
    minecraft_path: PathBuf,
    builds: Vec<String>,
    profile: String,
}

impl Runner {
    pub fn new(
        config: PathBuf,
        minecraft_path: PathBuf,
        builds: Vec<String>,
        profile: String,
    ) -> Self {
        Self {
            config,
            minecraft_path,
            builds,
            profile,
        }
    }

    pub fn start_standard(self) -> anyhow::Result<()> {
        let mut compilers = self.create_compilers()?;

        compilers.par_iter_mut().for_each(PackCompiler::run);

        Ok(())
    }

    pub fn start_listener(self) -> anyhow::Result<()> {
        self.listener_run();

        let watcher_config = notify::Config::default().with_poll_interval(Duration::from_secs(2));

        let config_path = self.config.clone();
        let source_path = self
            .config
            .parent()
            .expect("Coudln't get parent folder of config.")
            .join("src");

        let mut watcher: INotifyWatcher = notify::Watcher::new(
            move |res: Result<notify::event::Event, _>| match res {
                Ok(event) => match event.kind {
                    notify::EventKind::Modify(_)
                    | notify::EventKind::Create(_)
                    | notify::EventKind::Remove(_) => self.listener_run(),
                    _ => (),
                },
                Err(e) => {
                    println!("File listener error: {}", e);
                }
            },
            watcher_config,
        )?;

        watcher.watch(&config_path, RecursiveMode::NonRecursive)?;
        watcher.watch(&source_path, RecursiveMode::Recursive)?;

        loop {}
    }

    fn listener_run(&self) {
        match self.create_compilers() {
            Ok(mut compilers) => compilers.par_iter_mut().for_each(PackCompiler::run),
            Err(e) => println!("Failed to create compilers: {}", e),
        }
    }

    fn create_compilers(&self) -> anyhow::Result<Vec<PackCompiler>> {
        let config_raw = std::fs::read_to_string(&self.config).context("Config read error.")?;
        let config = toml::from_str::<PackConfig>(&config_raw).context("Config parse error.")?;

        let mut asset_tracker = AssetTracker::default();

        asset_tracker.search_bundle_files(&Path::new("./src").canonicalize()?)?;

        let asset_tracker = Arc::from(asset_tracker);

        let mut compilers = Vec::with_capacity(self.builds.len());
        let profile = config.get_profile(&self.profile)?;

        for build_name in &self.builds {
            let build = config.get_build(build_name)?.clone();
            let pack = config.condence_packs(&build.pack, &profile.pack);
            let compiler = PackCompiler::new(
                self.config
                    .parent()
                    .with_context(|| {
                        format!("Failed to get project path for build: {}", build_name)
                    })?
                    .to_owned(),
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
