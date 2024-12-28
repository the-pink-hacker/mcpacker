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
    version_name: String,
    version_number: String,
}

impl DeployAPIContext {
    pub fn new(
        modrinth_api_token: impl AsRef<str>,
        version_name: String,
        version_number: String,
    ) -> anyhow::Result<Self> {
        Ok(DeployAPIContext {
            modrinth: Ferinth::new(
                env!("CARGO_CRATE_NAME"),
                Some(env!("CARGO_PKG_VERSION")),
                Some("pink@thepinkhacker.com"),
                Some(modrinth_api_token.as_ref()),
            )?,
            version_name,
            version_number,
        })
    }
}

impl PackCompiler<'_> {
    pub async fn deploy(
        self,
        api_context: &DeployAPIContext,
        changelog: impl AsRef<str>,
    ) -> anyhow::Result<()> {
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

            let mut zip_name_part = zip_name
                .chars()
                .filter(char::is_ascii)
                .collect::<String>()
                .replace(' ', "-");
            zip_name_part += "-primary";

            let file_contents = std::fs::read(zip_path)?;
            let version_meta = self.as_modrinth_version(
                api_context,
                zip_name_part.clone(),
                changelog.as_ref().to_string(),
            )?;
            let response = api_context
                .modrinth
                .create_version(
                    &version_meta,
                    vec![(zip_name_part, zip_name, file_contents)],
                )
                .await?;

            println!("{:#?}", response);
        }

        Ok(())
    }

    pub fn as_modrinth_version(
        self,
        api_context: &DeployAPIContext,
        file_name: String,
        changelog: String,
    ) -> anyhow::Result<CreateVersion> {
        let primary_version = self
            .minecraft_versions
            .first()
            .context("There must be at least one Minecraft version supplied.")?;
        let name = format!("{} - {}", api_context.version_name, primary_version);
        let version_number = format!("{}-{}", api_context.version_number, primary_version);
        Ok(CreateVersion {
            name,
            version_number,
            changelog: Some(changelog),
            dependencies: self.pack.modrinth_dependencies,
            game_versions: self.minecraft_versions,
            version_type: VersionType::Alpha,
            loaders: vec!["minecraft".to_string()],
            featured: true,
            status: Some(Status::Listed),
            requested_status: Some(RequestedStatus::Listed),
            project_id: self
                .pack
                .modrinth_project_id
                .context("Missing pack field `modrinth_project_id` from pack.toml.")?,
            file_parts: vec![file_name],
            primary_file: None,
        })
    }
}
