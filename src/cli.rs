use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::config::PackConfig;

#[derive(Debug, Subcommand, Clone)]
pub enum Subcommands {
    /// Build one or many packs from the config.
    Build {
        /// Which profile should be used to build the pack(s).
        profile: String,

        /// The pack(s) that should be built.
        build: String,
    },
}

/// A CLI for packing and distributing Minecraft resource packs.
#[derive(Debug, Parser, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The config file used to build the resource pack.
    #[arg(short, long, value_name = "FILE", default_value = PathBuf::from("./pack.toml").into_os_string())]
    pub config: PathBuf,

    /// The minecraft directory.
    #[arg(short, long, value_name = "PATH", default_value = PathBuf::from("~/.minecraft/").into_os_string())]
    pub minecraft: PathBuf,

    /// The path where packs are compiled.
    #[arg(long, value_name = "PATH", default_value = PathBuf::from("./build/").into_os_string())]
    pub compile: PathBuf,

    #[command(subcommand)]
    pub commands: Subcommands,
}

impl Args {
    pub fn parse_config(&self) -> anyhow::Result<PackConfig> {
        let data = std::fs::read_to_string(&self.config)?;
        Ok(toml::from_str(&data)?)
    }
}
