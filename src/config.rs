pub mod export;

use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;

use crate::cli::Args;

use self::export::{ExportOutputType, ExportRelocation, JsonExportType, PackCompiler};

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
    #[serde(default = "PackMetaConfig::default_icon_path")]
    icon: PathBuf,
}

impl PackMetaConfig {
    fn default_icon_path() -> PathBuf {
        PathBuf::from("./pack.png")
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct ProfileConfig {
    pub output_type: ExportOutputType,
    pub relocation: ExportRelocation,
    pub json_type: JsonExportType,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct CollectionConfig {
    pub pack: Option<PackMetaConfig>,
    pub bundles: Vec<PathBuf>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct PackConfig {
    pack: PackMetaConfig,
    profile: HashMap<String, ProfileConfig>,
    build: HashMap<String, CollectionConfig>,
}

impl PackConfig {
    pub fn build_packs(&self, args: &Args, profile: &str, build: &str) -> anyhow::Result<()> {
        let profile = self.profile.get(profile).expect("Couldn't find profile.");
        let build = self.build.get(build).expect("Couldn't find build.");

        let name = self.pack.name.clone().unwrap_or_default();

        let compile_path = args.compile.join(&name);

        let compiler = PackCompiler::from(
            compile_path,
            args.minecraft.join("resourcepacks").join(name),
            &self.pack,
            profile,
            build,
        );
        compiler.run()?;

        Ok(())
    }
}
