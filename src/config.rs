pub mod export;

use std::{
    collections::{BTreeSet, HashMap},
    path::PathBuf,
    sync::Arc,
};

use anyhow::Context;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, OneOrMany};

use crate::{
    asset::zfighting::ZFightingModifier, compile::redirect::Redirect,
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
            Self::Single(format) => *format,
            Self::Range {
                minimum,
                maximum: _,
            } => *minimum,
            Self::List(formats) => *formats.first().expect("Format list was empty."),
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
    pub modrinth_project_id: Option<String>,
    pub seed: Option<u64>,
    pub zfighting_modifiers: Option<Vec<ZFightingModifier>>,
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
            modrinth_project_id: Self::condence_option(
                global.modrinth_project_id,
                build.modrinth_project_id,
                profile.modrinth_project_id,
            ),
            seed: Self::condence_option(global.seed, build.seed, profile.seed),
            zfighting_modifiers: Self::condence_option(
                global.zfighting_modifiers,
                build.zfighting_modifiers,
                profile.zfighting_modifiers,
            ),
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

#[serde_as]
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct CollectionConfig {
    pub pack: PackMetaConfig,
    #[serde_as(as = "OneOrMany<_>")]
    #[serde(default)]
    pub bundles: Vec<PathBuf>,
    #[serde_as(as = "OneOrMany<_>")]
    #[serde(default)]
    pub redirects: Vec<PathBuf>,
    #[serde_as(as = "OneOrMany<_>")]
    #[serde(default)]
    pub minecraft_versions: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedirectFile {
    pub redirect: Redirect,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct PackConfig {
    pub pack: PackMetaConfig,
    pub profile: HashMap<String, ProfileConfig>,
    pub build: IndexMap<String, CollectionConfig>,
}

impl PackConfig {
    pub fn get_profile(&self, profile: &str) -> anyhow::Result<Arc<ProfileConfig>> {
        self.profile
            .get(profile)
            .map(|p| Arc::new(p.clone()))
            .with_context(|| format!("Couldn't find profile: {}", profile))
    }

    pub fn get_build<'a>(&'a self, build: &str) -> anyhow::Result<&'a CollectionConfig> {
        self.build
            .get(build)
            .with_context(|| format!("Couldn't find build: {}", build))
    }

    pub fn condence_packs(
        &self,
        build: &PackMetaConfig,
        profile: &PackMetaConfig,
    ) -> PackMetaConfig {
        PackMetaConfig::condence(self.pack.clone(), build.clone(), profile.clone())
    }
}
