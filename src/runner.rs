use std::{path::PathBuf, time::Duration};

use anyhow::Context;
use notify::{INotifyWatcher, RecursiveMode, Watcher};
use rayon::prelude::*;

use crate::config::{export::PackCompiler, PackConfig};

pub struct Runner {
    config_path: PathBuf,
    compile_path: PathBuf,
    minecraft_path: PathBuf,
    builds: Vec<String>,
    profile: String,
}

impl Runner {
    pub fn new(
        config_path: PathBuf,
        compile_path: PathBuf,
        minecraft_path: PathBuf,
        builds: Vec<String>,
        profile: String,
    ) -> Self {
        Self {
            config_path,
            compile_path,
            minecraft_path,
            builds,
            profile,
        }
    }

    pub fn start_standard(self) -> anyhow::Result<()> {
        let compilers = self.create_compilers()?;

        compilers.par_iter().for_each(|compiler| compiler.run());

        Ok(())
    }

    pub fn start_listener(self) -> anyhow::Result<()> {
        self.listener_run();

        let watcher_config = notify::Config::default().with_poll_interval(Duration::from_secs(2));

        let config_path = self.config_path.clone();
        let source_path = self
            .config_path
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
        self.create_compilers()
            .unwrap()
            .par_iter()
            .for_each(PackCompiler::run)
    }

    fn create_compilers(&self) -> anyhow::Result<Vec<PackCompiler>> {
        let config_raw =
            std::fs::read_to_string(&self.config_path).context("Config read error.")?;
        let config = toml::from_str::<PackConfig>(&config_raw).context("Config parse error.")?;

        let mut compilers = Vec::with_capacity(self.builds.len());
        let profile = config.find_profile(&self.profile);

        for build in &self.builds {
            let compiler = config.create_compiler(
                &self.compile_path,
                &self.minecraft_path,
                profile.clone(),
                &build,
            );
            compilers.push(compiler);
        }

        Ok(compilers)
    }
}
