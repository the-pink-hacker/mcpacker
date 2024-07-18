use anyhow::Context;
use ferinth::{
    structures::version::{CreateVersion, RequestedStatus, Status, VersionType},
    Ferinth,
};

use crate::config::export::ExportOutputType;

use super::PackCompiler;

#[derive(Default)]
pub struct DeployAPIContext {
    modrinth: Ferinth,
}

impl DeployAPIContext {
    pub fn new(modrinth_api_token: &str) -> anyhow::Result<Self> {
        Ok(DeployAPIContext {
            modrinth: Ferinth::new(
                env!("CARGO_CRATE_NAME"),
                Some(env!("CARGO_PKG_VERSION")),
                Some("pink@thepinkhacker.com"),
                Some(modrinth_api_token),
            )?,
        })
    }
}

impl<'a> PackCompiler<'a> {
    pub async fn deploy(&self, api_context: &DeployAPIContext) -> anyhow::Result<()> {
        if self.profile.output_type == ExportOutputType::Zip {
            let zip_path = Self::get_zip_path(&self.compile_path);
            let zip_name = zip_path
                .file_name()
                .with_context(|| {
                    format!(
                        "Failed to get zip file's name located at: {}",
                        zip_path.display()
                    )
                })?
                .to_string_lossy()
                .to_string();
            let file_contents = std::fs::read(zip_path)?;
            let version_meta = self.as_modrinth_version(zip_name.clone())?;
            let response = api_context
                .modrinth
                .create_version(&version_meta, vec![(zip_name, file_contents)])
                .await?;

            println!("{:#?}", response);
        }

        Ok(())
    }

    pub fn as_modrinth_version(&self, file_name: String) -> anyhow::Result<CreateVersion> {
        Ok(CreateVersion {
            name: "API TEST".to_string(),
            version_number: "test2".to_string(),
            changelog: Some("# **this is an API test**".to_string()),
            dependencies: vec![],
            game_versions: vec!["1.9".to_string()],
            version_type: VersionType::Alpha,
            loaders: vec!["minecraft".to_string()],
            featured: false,
            status: Some(Status::Listed),
            requested_status: Some(RequestedStatus::Listed),
            project_id: self
                .pack
                .modrinth_project_id
                .clone()
                .context("Missing pack field `modrinth_project_id` from pack.toml.")?,
            file_parts: vec![file_name],
            primary_file: None,
        })
    }
}
