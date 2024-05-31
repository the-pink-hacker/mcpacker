use std::{
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use serde::{Deserialize, Serialize};
use serde_json::ser::PrettyFormatter;
use zip::{write::SimpleFileOptions, ZipWriter};

use super::{CollectionConfig, FormatType, PackMetaConfig, ProfileConfig};

#[derive(Debug, Serialize, Default)]
struct PackMCMetaContents {
    pack_format: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    supported_formats: Option<FormatType>,
}

#[derive(Debug, Serialize, Default)]
pub struct PackMCMeta {
    pack: PackMCMetaContents,
}

impl PackMCMeta {
    fn from(pack: &PackMetaConfig) -> Self {
        let (pack_format, supported_formats) =
            pack.format.clone().unwrap_or_default().get_formats();

        Self {
            pack: PackMCMetaContents {
                description: pack.description.clone(),
                pack_format,
                supported_formats,
            },
        }
    }
}

#[derive(Debug, Deserialize, Default, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExportOutputType {
    #[default]
    Uncompressed,
    Zip,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExportRelocation {
    #[default]
    None,
    Copy,
    Symbolic,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "snake_case")]
pub enum JsonExportType {
    #[default]
    Compact,
    Pretty,
}

impl JsonExportType {
    pub fn to_string<T>(&self, value: &T) -> anyhow::Result<String>
    where
        T: ?Sized + Serialize,
    {
        match self {
            Self::Compact => Ok(serde_json::to_string(value)?),
            Self::Pretty => {
                let formatter = PrettyFormatter::with_indent(b"    ");
                let mut writter = Vec::new();
                let mut serializer =
                    serde_json::ser::Serializer::with_formatter(&mut writter, formatter);

                value.serialize(&mut serializer)?;
                Ok(String::from_utf8(writter)?)
            }
        }
    }
}

pub struct PackCompiler {
    compile_path: PathBuf,
    asset_path: PathBuf,
    bundles_path: PathBuf,
    resourcepack_path: PathBuf,
    pack: PackMetaConfig,
    profile: Arc<ProfileConfig>,
    build: CollectionConfig,
}

const PACK_META_NAME: &str = "pack.mcmeta";
const PACK_ICON_NAME: &str = "pack.png";

impl PackCompiler {
    pub fn from(
        compile_path: PathBuf,
        minecraft_path: PathBuf,
        pack: PackMetaConfig,
        profile: Arc<ProfileConfig>,
        build: CollectionConfig,
    ) -> Self {
        let mut name = pack.name.clone().unwrap_or_default();

        if let Some(suffix) = &pack.suffix {
            name += suffix;
        }

        let compile_path = compile_path.join(&name);

        Self {
            pack,
            profile,
            build,
            bundles_path: PathBuf::from("src"),
            asset_path: compile_path.join("assets"),
            resourcepack_path: minecraft_path.join("resourcepacks").join(name),
            compile_path,
        }
    }

    pub fn run(&self) {
        println!("Compiling...");

        let current_time = Instant::now();

        match self.run_failable() {
            Ok(_) => {
                let time_passed = current_time.elapsed();
                println!("Completed in {:.2} seconds.", time_passed.as_secs_f32());
            }
            Err(e) => println!("Build error: {:}", e),
        }
    }

    fn run_failable(&self) -> anyhow::Result<()> {
        self.setup_compile_path()?;
        self.compile_meta()?;
        self.compile_icon()?;

        for bundle in &self.build.bundles {
            self.compile_bundle(bundle)?;
        }

        self.output()?;
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
        let meta = PackMCMeta::from(&self.pack);
        let raw = self.profile.json_type.to_string(&meta)?;
        let mut file = File::create(&self.compile_path.join(PACK_META_NAME))?;
        file.write_all(raw.as_bytes())?;

        Ok(())
    }

    fn compile_icon(&self) -> std::io::Result<()> {
        std::fs::copy(
            &self
                .pack
                .icon
                .clone()
                .unwrap_or_else(|| PathBuf::from("./pack.png")),
            &self.compile_path.join(PACK_ICON_NAME),
        )?;
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
        let expanded_pack_path = shellexpand::path::tilde(&self.resourcepack_path);
        let (expanded_source, expanded_link) = match self.profile.output_type {
            ExportOutputType::Zip => (
                self.compile_path.with_extension("zip").canonicalize()?,
                expanded_pack_path.with_extension("zip"),
            ),
            ExportOutputType::Uncompressed => (
                self.compile_path.canonicalize()?,
                expanded_pack_path.to_path_buf(),
            ),
        };

        if expanded_link.exists() {
            return Ok(());
        }

        symlink::symlink_auto(expanded_source, expanded_link)?;

        Ok(())
    }

    fn output(&self) -> anyhow::Result<()> {
        match self.profile.output_type {
            ExportOutputType::Zip => self.zip(),
            ExportOutputType::Uncompressed => Ok(()),
        }
    }

    fn zip(&self) -> anyhow::Result<()> {
        let zip_path = self.compile_path.with_extension("zip");
        let zip_file = File::create(zip_path)?;
        let mut zip = ZipWriter::new(zip_file);
        let zip_options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        let files_paths = glob::glob(
            self.compile_path
                .join("**")
                .join("*")
                .to_str()
                .expect("Couldn't convert path to unicode."),
        )
        .unwrap()
        .filter_map(Result::ok)
        .filter(|f| f.is_file())
        .collect::<Vec<_>>();

        for file_path in &files_paths {
            let file = File::open(file_path)?;
            let absolute_path = file_path.canonicalize()?;
            let file_zip_path = absolute_path
                .strip_prefix(self.compile_path.canonicalize()?)?
                .to_str()
                .unwrap();

            zip.start_file(file_zip_path, zip_options)?;

            let mut buffer = Vec::new();
            io::copy(&mut file.take(u64::MAX), &mut buffer)?;

            zip.write_all(&buffer)?;
        }

        zip.finish()?;

        Ok(())
    }
}
