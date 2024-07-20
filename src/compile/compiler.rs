use std::{
    fs::File,
    io::Write,
    io::{self, Read},
    path::{Path, PathBuf},
    time::Instant,
};

use walkdir::WalkDir;
use zip::{write::SimpleFileOptions, ZipWriter};

use crate::{
    config::{
        export::{ExportOutputType, ExportRelocation, PackMCMeta},
        RedirectFile,
    },
    minecraft::asset::types::identifier::AssetType,
};

use super::{library::CompiledAssetLibrary, PackCompiler};

const PACK_META_NAME: &str = "pack.mcmeta";
const PACK_ICON_NAME: &str = "pack.png";

impl<'a> PackCompiler<'a> {
    pub async fn run(mut self) -> Self {
        println!("Compiling...");

        let current_time = Instant::now();

        match self.run_failable() {
            Ok(_) => {
                let time_passed = current_time.elapsed();
                println!("Completed in {:.2} seconds.", time_passed.as_secs_f32());
            }
            Err(e) => println!("Build error: {:}", e),
        }

        self
    }

    pub fn get_bundle_path<P: AsRef<Path>>(&self, bundle: P) -> anyhow::Result<PathBuf> {
        self.project_sanitizer
            .sanitize(self.bundles_path.join(bundle))
    }

    pub fn get_redirect_path<P: AsRef<Path>>(&self, redirect: P) -> anyhow::Result<PathBuf> {
        self.project_sanitizer
            .sanitize(self.redirects_path.join(redirect).with_extension("toml"))
    }

    fn run_failable(&mut self) -> anyhow::Result<()> {
        let mut library = self.populate_asset_library()?.compile(self)?;

        self.process_redirects(&mut library)?;

        self.setup_compile_path()?;
        self.compile_meta()?;
        self.compile_icon()?;
        self.compile_license()?;

        library.write_contents(&self)?;

        self.output()?;
        self.relocate()?;

        Ok(())
    }

    fn process_redirects(&mut self, library: &mut CompiledAssetLibrary) -> anyhow::Result<()> {
        for redirect_path in &self.redirects {
            let raw = std::fs::read_to_string(self.get_redirect_path(redirect_path)?)?;
            let redirect = toml::from_str::<RedirectFile>(&raw)?.redirect;

            match redirect.asset_type {
                AssetType::Texture => {
                    for model in library.models.values_mut() {
                        model.apply_texture_redirect(&redirect);
                    }
                }
                _ => unimplemented!("Asset type not supported."),
            }
        }

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

    fn compile_icon(&self) -> anyhow::Result<()> {
        if let Some(icon) = &self.pack.icon {
            std::fs::copy(
                self.project_sanitizer.join(icon)?,
                &self.compile_path.join(PACK_ICON_NAME),
            )?;
        }
        Ok(())
    }

    fn compile_license(&self) -> anyhow::Result<()> {
        if let Some(license) = &self.pack.license {
            let file_name = license.file_name().unwrap_or_default();
            std::fs::copy(
                self.project_sanitizer.join(license)?,
                &self.compile_path.join(file_name),
            )?;
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
                Self::get_zip_path(&self.compile_path),
                Self::get_zip_path(expanded_pack_path),
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

    pub fn get_zip_path(path: impl Into<PathBuf>) -> PathBuf {
        let mut path = path.into();
        path.as_mut_os_string().push(".zip");
        path
    }

    fn zip(&self) -> anyhow::Result<()> {
        let zip_path = Self::get_zip_path(&self.compile_path);
        let zip_file = File::create(zip_path)?;
        let mut zip = ZipWriter::new(zip_file);
        let zip_options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for file_path in WalkDir::new(&self.compile_path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|f| f.path().is_file())
        {
            let file_path = file_path.path().canonicalize()?;
            let file = File::open(&file_path)?;
            let file_zip_path = file_path
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
