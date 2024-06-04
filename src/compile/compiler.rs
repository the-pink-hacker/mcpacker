use std::{
    fs::File,
    io::Write,
    io::{self, Read},
    path::PathBuf,
    time::Instant,
};

use zip::{write::SimpleFileOptions, ZipWriter};

use crate::config::export::{ExportOutputType, ExportRelocation, PackMCMeta};

use super::PackCompiler;

const PACK_META_NAME: &str = "pack.mcmeta";
const PACK_ICON_NAME: &str = "pack.png";

impl PackCompiler {
    pub fn run(&mut self) {
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

    fn run_failable(&mut self) -> anyhow::Result<()> {
        self.populate_asset_library()?;

        self.setup_compile_path()?;
        self.compile_meta()?;
        self.compile_icon()?;
        self.write_asset_library()?;

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
                .unwrap_or_else(|| PathBuf::from(PACK_ICON_NAME)),
            &self.compile_path.join(PACK_ICON_NAME),
        )?;
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

    fn get_zip_path(path: impl Into<PathBuf>) -> PathBuf {
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
