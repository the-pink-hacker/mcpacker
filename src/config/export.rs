use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use super::{CollectionConfig, PackMetaConfig, ProfileConfig};

#[derive(Debug, Serialize, Default)]
struct PackMCMetaContents {
    pack_format: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct PackMCMeta {
    pack: PackMCMetaContents,
}

impl PackMCMeta {
    fn basic(pack: &PackMetaConfig) -> Self {
        let pack_formats: Vec<_> = pack.format.clone().into();

        Self {
            pack: PackMCMetaContents {
                pack_format: *pack_formats
                    .iter()
                    .min()
                    .expect("Failed to get minimum pack format version."),
                description: pack.description.clone(),
            },
        }
    }
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
#[serde(rename_all = "snake_case")]
pub enum JsonExportType {
    #[default]
    Compact,
    Pretty,
}

impl JsonExportType {
    pub fn to_string<T>(&self, value: &T) -> serde_json::Result<String>
    where
        T: ?Sized + Serialize,
    {
        match self {
            Self::Compact => serde_json::to_string(value),
            Self::Pretty => serde_json::to_string_pretty(value),
        }
    }
}

pub struct PackCompiler<'a, 'b, 'c> {
    compile_path: PathBuf,
    asset_path: PathBuf,
    bundles_path: PathBuf,
    resourcepack_path: PathBuf,
    pack: &'a PackMetaConfig,
    profile: &'b ProfileConfig,
    build: &'c CollectionConfig,
}

const PACK_META_NAME: &str = "pack.mcmeta";
const PACK_ICON_NAME: &str = "pack.png";

impl<'a, 'b, 'c> PackCompiler<'a, 'b, 'c> {
    pub fn from(
        compile_path: PathBuf,
        resourcepack_path: PathBuf,
        pack: &'a PackMetaConfig,
        profile: &'b ProfileConfig,
        build: &'c CollectionConfig,
    ) -> Self {
        Self {
            pack,
            profile,
            build,
            bundles_path: PathBuf::from("src"),
            asset_path: compile_path.join("assets"),
            resourcepack_path,
            compile_path,
        }
    }

    pub fn run(&self) -> anyhow::Result<()> {
        self.setup_compile_path()?;
        self.compile_meta()?;
        self.compile_icon()?;

        for bundle in &self.build.bundles {
            self.compile_bundle(bundle)?;
        }

        self.relocate()?;

        Ok(())
    }

    fn setup_compile_path(&self) -> std::io::Result<()> {
        if self.compile_path.exists() {
            std::fs::remove_dir_all(&self.compile_path)?;
        }

        std::fs::create_dir_all(&self.compile_path)?;
        Ok(())
    }

    fn compile_meta(&self) -> anyhow::Result<()> {
        let meta = PackMCMeta::basic(&self.pack);
        let raw = self.profile.json_type.to_string(&meta)?;
        let mut file = File::create(&self.compile_path.join(PACK_META_NAME))?;
        file.write_all(raw.as_bytes())?;

        Ok(())
    }

    fn compile_icon(&self) -> std::io::Result<()> {
        std::fs::copy(&self.pack.icon, &self.compile_path.join(PACK_ICON_NAME))?;
        Ok(())
    }

    fn compile_bundle(&self, bundle: &Path) -> anyhow::Result<()> {
        let src_path = self.bundles_path.join(bundle);
        let files = glob::glob(
            src_path
                .join("**")
                .join("*")
                .to_str()
                .expect("Couldn't convert path to unicode."),
        )
        .unwrap()
        .filter_map(Result::ok)
        .filter(|f| f.is_file());

        for file in files {
            let file_absolute = file.canonicalize()?;
            let relative_file = file_absolute.strip_prefix(&src_path.canonicalize()?)?;
            let to = self.asset_path.join(relative_file);
            let mut to_path = to.clone();
            to_path.pop();
            std::fs::create_dir_all(&to_path)?;
            std::fs::copy(file, to)?;
        }

        Ok(())
    }

    fn relocate(&self) -> anyhow::Result<()> {
        match self.profile.relocation {
            ExportRelocation::None => (),
            ExportRelocation::Copy => unimplemented!("Copy relocation."),
            ExportRelocation::Symbolic => self.relocate_symbolic()?,
        }

        Ok(())
    }

    fn relocate_symbolic(&self) -> anyhow::Result<()> {
        let expanded_link = shellexpand::path::tilde(&self.resourcepack_path);

        if expanded_link.exists() {
            return Ok(());
        }

        let compile_path_absolute = self.compile_path.canonicalize()?;
        symlink::symlink_dir(compile_path_absolute, expanded_link)?;
        Ok(())
    }
}
