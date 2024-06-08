pub mod export;

use std::{
    collections::{BTreeSet, HashMap},
    path::PathBuf,
    sync::Arc,
};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{
    compile::{tracking::AssetTracker, PackCompiler},
    minecraft::asset::types::text::RawText,
};

use self::export::{ExportOutputType, ExportRelocation, JsonExportType};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum FormatType {
    Single(u8),
    Range {
        #[serde(alias = "minimum", rename = "min_inclusive")]
        minimum: u8,
        #[serde(alias = "maximum", rename = "max_inclusive")]
        maximum: u8,
    },
    List(BTreeSet<u8>),
}

impl FormatType {
    pub fn min(&self) -> u8 {
        match self {
            Self::Single(format) => format.clone(),
            Self::Range {
                minimum,
                maximum: _,
            } => minimum.clone(),
            Self::List(formats) => formats.first().expect("Format list was empty.").clone(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Single(_) => 1,
            Self::Range { minimum, maximum } => (maximum - minimum) as usize,
            Self::List(formats) => formats.len(),
        }
    }

    pub fn get_formats(self) -> (u8, Option<Self>) {
        let minimum = self.min();

        let formats = match self {
            Self::Single(_) => None,
            _ => Some(self),
        };

        (minimum, formats)
    }
}

impl Default for FormatType {
    fn default() -> Self {
        Self::Single(1)
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PackMetaConfig {
    pub name: Option<RawText>,
    pub suffix: Option<RawText>,
    pub description: Option<RawText>,
    pub format: Option<FormatType>,
    pub icon: Option<PathBuf>,
    pub license: Option<PathBuf>,
}

impl PackMetaConfig {
    pub fn condence(global: Self, build: Self, profile: Self) -> Self {
        let mut name = Self::condence_option(global.name, build.name, profile.name);

        Self::append_name(&mut name, profile.suffix);
        Self::append_name(&mut name, build.suffix);
        Self::append_name(&mut name, global.suffix);

        Self {
            name,
            suffix: None,
            description: Self::condence_option(
                global.description,
                build.description,
                profile.description,
            ),
            format: Self::condence_option(global.format, build.format, profile.format),
            icon: Self::condence_option(global.icon, build.icon, profile.icon),
            license: Self::condence_option(global.license, build.license, profile.license),
        }
    }

    fn append_name(name: &mut Option<RawText>, suffix: Option<RawText>) {
        if let Some(name) = name {
            if let Some(suffix) = suffix {
                *name += suffix;
            }
        } else {
            *name = suffix;
        }
    }

    fn condence_option<T>(global: Option<T>, build: Option<T>, profile: Option<T>) -> Option<T> {
        if profile.is_some() {
            profile
        } else if build.is_some() {
            build
        } else {
            global
        }
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct ProfileConfig {
    pub output_type: ExportOutputType,
    pub relocation: ExportRelocation,
    pub json_type: JsonExportType,
    pub pack: PackMetaConfig,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct CollectionConfig {
    pub pack: PackMetaConfig,
    pub bundles: Vec<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PackConfig {
    pack: PackMetaConfig,
    profile: HashMap<String, ProfileConfig>,
    build: HashMap<String, CollectionConfig>,
}

impl PackConfig {
    pub fn find_profile(&self, profile: &str) -> anyhow::Result<Arc<ProfileConfig>> {
        let profile = self
            .profile
            .get(profile)
            .with_context(|| format!("Couldn't find profile: {}", profile))?;

        Ok(Arc::new(profile.clone()))
    }

    pub fn create_compiler(
        &self,
        compile_path: &PathBuf,
        minecraft_path: &PathBuf,
        profile: Arc<ProfileConfig>,
        build: &str,
        tracker: Arc<AssetTracker>,
    ) -> PackCompiler {
        let build = self.build.get(build).expect("Couldn't find build.");
        let pack =
            PackMetaConfig::condence(self.pack.clone(), build.pack.clone(), profile.pack.clone());

        PackCompiler::from(
            compile_path.clone(),
            minecraft_path.clone(),
            pack,
            profile,
            build.clone(),
            tracker,
        )
    }
}
