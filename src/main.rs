use std::path::PathBuf;

use clap::Parser;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PackMetaConfig {
    name: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PackConfig {
    meta: Option<PackMetaConfig>,
}

#[derive(Debug, Deserialize)]
pub struct BuildConfig {
    pack: Option<PackConfig>,
}

/// A CLI for packing and distributing Minecraft resource packs.
#[derive(Debug, Parser)]
#[command(about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE", default_value = PathBuf::from("./pack.toml").into_os_string())]
    /// The config file used to build the resource pack.
    config: PathBuf,

    #[arg(short, long, value_name = "PATH", default_value = PathBuf::from("~/.minecraft/").into_os_string())]
    /// The minecraft directory.
    minecraft: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let config_raw = std::fs::read_to_string(args.config)?;

    let config = toml::from_str::<BuildConfig>(&config_raw)?;

    println!("{:?}", config);

    Ok(())
}
