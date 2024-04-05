use std::{collections::HashMap, fs::File, io::Write, path::PathBuf};

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum FormatType {
    Single(u8),
    Range { minimum: u8, maximum: u8 },
    List(Vec<u8>),
}

impl From<FormatType> for Vec<u8> {
    fn from(value: FormatType) -> Self {
        match value {
            FormatType::Single(raw_value) => vec![raw_value],
            FormatType::Range { minimum, maximum } => (minimum..maximum).collect::<Vec<_>>(),
            FormatType::List(raw_values) => raw_values,
        }
    }
}

impl Default for FormatType {
    fn default() -> Self {
        Self::Single(1)
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct PackMetaConfig {
    name: Option<String>,
    description: Option<String>,
    format: FormatType,
}

#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExportOutputType {
    #[default]
    Uncompressed,
    Zip,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExportRelocation {
    #[default]
    None,
    Copy,
    Symbolic,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct ProfileConfig {
    output_type: ExportOutputType,
    relocation: ExportRelocation,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct CollectionConfig {
    pack: PackMetaConfig,
    bundles: Vec<PathBuf>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct PackConfig {
    pack: PackMetaConfig,
    profile: HashMap<String, ProfileConfig>,
    build: HashMap<String, CollectionConfig>,
}

const PACK_META_NAME: &str = "pack.mcmeta";

impl PackConfig {
    pub fn build_packs(&self, args: &Args, profile: &str, build: &str) -> anyhow::Result<()> {
        let profile = self.profile.get(profile).expect("Couldn't find profile.");
        let build = self.build.get(build).expect("Couldn't find build.");

        let name = self.pack.name.clone().unwrap_or_default();

        let compile_path = args.compile.join(&name);

        if compile_path.exists() {
            std::fs::remove_dir_all(&compile_path)?;
        }

        std::fs::create_dir_all(&compile_path)?;

        let description = self.pack.description.clone().unwrap_or_default();
        let pack_formats: Vec<_> = self.pack.format.clone().into();
        let pack_format = pack_formats
            .iter()
            .min()
            .expect("Failed to get minimum pack format version.");

        let meta = PackMCMeta::new(pack_format.clone(), description);
        let meta_output = serde_json::to_string(&meta)?;
        let mut meta_file = File::create(&compile_path.join(PACK_META_NAME))?;
        meta_file.write_all(meta_output.as_bytes())?;

        let bundles_path = PathBuf::from("./src/");
        let asset_path = compile_path.join("assets");

        for bundle in &build.bundles {
            let bundle_path = bundles_path.join(bundle);
            let files = glob::glob(
                bundle_path
                    .join("**")
                    .join("*")
                    .to_str()
                    .expect("Coudln't convert path to unicode."),
            )
            .unwrap()
            .filter_map(Result::ok)
            .filter(|f| f.is_file());

            for file in files {
                let to = asset_path.join(&file);
                let mut to_path = to.clone();
                to_path.pop();
                std::fs::create_dir_all(&to_path)?;
                std::fs::copy(file, to)?;
            }
        }

        match profile.relocation {
            ExportRelocation::None => (),
            ExportRelocation::Copy => unimplemented!("Copy relocation"),
            ExportRelocation::Symbolic => {
                let link = args.minecraft.join("resourcepacks").join(name);
                let expanded_link = shellexpand::path::tilde(&link);
                if !expanded_link.exists() {
                    let compile_path_absolute = compile_path.canonicalize()?;
                    symlink::symlink_dir(compile_path_absolute, expanded_link)?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Default)]
struct PackMCMetaContents {
    pack_format: u8,
    #[serde(skip_serializing_if = "String::is_empty")]
    description: String,
}

#[derive(Debug, Serialize, Default)]
pub struct PackMCMeta {
    pack: PackMCMetaContents,
}

impl PackMCMeta {
    pub fn new(pack_format: u8, description: String) -> Self {
        Self {
            pack: PackMCMetaContents {
                pack_format,
                description,
            },
        }
    }
}

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
    config: PathBuf,

    /// The minecraft directory.
    #[arg(short, long, value_name = "PATH", default_value = PathBuf::from("~/.minecraft/").into_os_string())]
    minecraft: PathBuf,

    /// The path where packs are compiled.
    #[arg(long, value_name = "PATH", default_value = PathBuf::from("./build/").into_os_string())]
    compile: PathBuf,

    #[command(subcommand)]
    commands: Subcommands,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let config_raw = std::fs::read_to_string(&args.config)?;
    let config = toml::from_str::<PackConfig>(&config_raw)?;

    println!("{:#?}", config);

    match &args.commands {
        Subcommands::Build { profile, build } => config.build_packs(&args, &profile, &build),
    }
}
