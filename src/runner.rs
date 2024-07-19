use std::{cell::Cell, collections::BTreeSet, path::PathBuf, sync::Arc, time::Duration};

use anyhow::Context;
use notify::{INotifyWatcher, RecursiveMode, Watcher};
use tokio::{sync::Mutex, task::JoinSet};

use crate::{
    compile::{deploy::DeployAPIContext, tracking::AssetTracker, PackCompiler},
    config::PackConfig,
    sanitize::PathSanitizer,
};

static POLL_RATE: Duration = Duration::from_secs(1);

pub struct Runner {
    project_sanitizer: PathSanitizer,
    config: PathBuf,
    minecraft_path: PathBuf,
    builds: Vec<String>,
    profile: String,
    api_context: Option<DeployAPIContext>,
    changed: Arc<Mutex<Cell<bool>>>,
}

impl Runner {
    pub fn build(
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
            api_context: None,
            changed: Arc::new(Mutex::new(Cell::new(true))),
        })
    }

    pub fn deploy(
        config: PathBuf,
        minecraft_path: PathBuf,
        builds: Vec<String>,
        profile: String,
        modrinth_api_token: &str,
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
            api_context: Some(DeployAPIContext::new(modrinth_api_token)?),
            changed: Arc::new(Mutex::new(Cell::new(true))),
        })
    }

    pub async fn run(&'static self) -> anyhow::Result<()> {
        let compilers = self.create_compilers()?;

        if let Some(api_context) = &self.api_context {
            for compiler in compilers {
                compiler.run().await.deploy(api_context).await?;
            }
        } else {
            let mut set = JoinSet::new();

            for compiler in compilers {
                set.spawn(compiler.run());
            }

            while let Some(res) = set.join_next().await {
                res?;
            }
        }

        Ok(())
    }

    pub async fn spawn_run_listener(&'static self) -> anyhow::Result<()> {
        self.spawn_file_watcher().await?;

        let mut compiler_listen_interval = tokio::time::interval(POLL_RATE);

        loop {
            compiler_listen_interval.tick().await;

            let mut changed = self.changed.lock().await;

            if changed.get() {
                *changed.get_mut() = false;
                self.run().await?;
            }
        }
    }

    async fn spawn_file_watcher(&'static self) -> anyhow::Result<()> {
        let watcher_config = notify::Config::default().with_compare_contents(true);

        let config_path = self.config.clone();
        let source_path = self.project_sanitizer.join("src")?;

        let mut watcher: INotifyWatcher = notify::Watcher::new(
            move |res: Result<notify::event::Event, _>| match res {
                Ok(event) => match event.kind {
                    notify::EventKind::Modify(_)
                    | notify::EventKind::Create(_)
                    | notify::EventKind::Remove(_) => {
                        tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .expect("Failed to create async file watcher runtime.")
                            .block_on(async move {
                                *self.changed.lock().await.get_mut() = true;
                            });
                    }
                    _ => (),
                },
                Err(e) => println!("File listener error: {}", e),
            },
            watcher_config,
        )?;

        watcher.watch(&config_path, RecursiveMode::NonRecursive)?;
        watcher.watch(&source_path, RecursiveMode::Recursive)?;

        tokio::spawn(async move {
            Self::break_off_watcher(watcher).await;
        });

        Ok(())
    }

    async fn break_off_watcher(_: INotifyWatcher) {
        loop {}
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
        let builds = self.filter_build_list(&config);

        for build_name in builds {
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

    fn filter_build_list<'a>(&'a self, config: &'a PackConfig) -> BTreeSet<&'a String> {
        let mut list = BTreeSet::from_iter(self.builds.iter());

        if list.contains(&"ALL".to_string()) {
            list = BTreeSet::from_iter(config.build.keys());
        }

        list
    }
}
