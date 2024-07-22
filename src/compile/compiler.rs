use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use async_fs::File;
use async_zip::{base::write::ZipFileWriter, Compression, ZipEntryBuilder};
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use walkdir::WalkDir;

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

        match self.run_failable().await {
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

    async fn run_failable(&mut self) -> anyhow::Result<()> {
        let mut library = self.populate_asset_library().await?.compile(self)?;

        self.process_redirects(&mut library).await?;

        self.setup_compile_path().await?;
        self.compile_meta().await?;
        self.compile_icon().await?;
        self.compile_license().await?;

        library.write_contents(&self).await?;

        self.output().await?;
        self.relocate()?;

        Ok(())
    }

    async fn process_redirects(
        &mut self,
        library: &mut CompiledAssetLibrary,
    ) -> anyhow::Result<()> {
        for redirect_path in &self.redirects {
            let raw = async_fs::read_to_string(self.get_redirect_path(redirect_path)?).await?;
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

    async fn setup_compile_path(&self) -> std::io::Result<()> {
        if self.compile_path.exists() {
            async_fs::remove_dir_all(&self.compile_path).await?;
        }

        async_fs::create_dir_all(&self.compile_path).await?;
        Ok(())
    }

    async fn compile_meta(&self) -> anyhow::Result<()> {
        let meta = PackMCMeta::from(&self.pack);
        let raw = self.profile.json_type.to_string(&meta)?;
        let mut file = File::create(&self.compile_path.join(PACK_META_NAME)).await?;
        file.write_all(raw.as_bytes()).await?;
        file.flush().await?;

        Ok(())
    }

    async fn compile_icon(&self) -> anyhow::Result<()> {
        if let Some(icon) = &self.pack.icon {
            async_fs::copy(
                self.project_sanitizer.join(icon)?,
                &self.compile_path.join(PACK_ICON_NAME),
            )
            .await?;
        }
        Ok(())
    }

    async fn compile_license(&self) -> anyhow::Result<()> {
        if let Some(license) = &self.pack.license {
            let file_name = license.file_name().unwrap_or_default();
            async_fs::copy(
                self.project_sanitizer.join(license)?,
                &self.compile_path.join(file_name),
            )
            .await?;
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

    async fn output(&self) -> anyhow::Result<()> {
        match self.profile.output_type {
            ExportOutputType::Zip => self.zip().await,
            ExportOutputType::Uncompressed => Ok(()),
        }
    }

    pub fn get_zip_path(path: impl Into<PathBuf>) -> PathBuf {
        let mut path = path.into();
        path.as_mut_os_string().push(".zip");
        path
    }

    async fn zip(&self) -> anyhow::Result<()> {
        let zip_path = Self::get_zip_path(&self.compile_path);
        let zip_file = File::create(zip_path).await?;
        let mut zip_writer = ZipFileWriter::new(zip_file);

        for file_path in WalkDir::new(&self.compile_path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|f| f.path().is_file())
        {
            let file_path = file_path.path().canonicalize()?;
            let mut file = File::open(&file_path).await?;
            let file_zip_path = file_path
                .strip_prefix(self.compile_path.canonicalize()?)?
                .to_str()
                .unwrap();

            let file_size = file.metadata().await?.len() as usize;

            let mut buffer = Vec::with_capacity(file_size);
            file.read_to_end(&mut buffer).await?;

            let builder = ZipEntryBuilder::new(file_zip_path.into(), Compression::Deflate);
            zip_writer.write_entry_whole(builder, &buffer).await?;
        }

        zip_writer.close().await?;

        Ok(())
    }
}
