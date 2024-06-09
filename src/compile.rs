use std::{path::PathBuf, sync::Arc};

use crate::config::{CollectionConfig, PackMetaConfig, ProfileConfig};

use self::{library::AssetLibrary, tracking::AssetTracker};

pub mod compiler;
pub mod library;
pub mod redirect;
pub mod tracking;

pub struct PackCompiler {
    compile_path: PathBuf,
    asset_path: PathBuf,
    bundles_path: PathBuf,
    resourcepack_path: PathBuf,
    pack: PackMetaConfig,
    profile: Arc<ProfileConfig>,
    build: CollectionConfig,
    library: AssetLibrary,
    tracker: Arc<AssetTracker>,
}

impl PackCompiler {
    pub fn from(
        compile_path: PathBuf,
        minecraft_path: PathBuf,
        pack: PackMetaConfig,
        profile: Arc<ProfileConfig>,
        build: CollectionConfig,
        tracker: Arc<AssetTracker>,
    ) -> Self {
        let name = pack.name.clone().unwrap_or_default().to_string();

        let compile_path = compile_path.join(&name);

        Self {
            pack,
            profile,
            build,
            bundles_path: PathBuf::from("./src")
                .canonicalize()
                .expect("Failed to get absolute bundle path."),
            asset_path: compile_path.join("assets"),
            resourcepack_path: minecraft_path.join("resourcepacks").join(name),
            compile_path,
            library: Default::default(),
            tracker,
        }
    }
}
