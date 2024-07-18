use std::sync::Mutex;

use anyhow::anyhow;
use ferinth::{
    structures::version::{CreateVersion, VersionType},
    Ferinth,
};

use super::PackCompiler;

#[derive(Default)]
pub struct DeployAPIContext {
    modrinth: Option<Ferinth>,
}

impl DeployAPIContext {
    fn init_modrinth<'a>(&'a mut self) -> anyhow::Result<&'a mut Ferinth> {
        let context = match self.modrinth {
            Some(ref mut context) => context,
            None => {
                self.modrinth = Some(Ferinth::new(
                    env!("CARGO_CRATE_NAME"),
                    Some(env!("CARGO_PKG_VERSION")),
                    Some("pink@thepinkhacker.com"),
                    Some(&std::env::var("MODRINTH_PERSONAL_TOKEN")?),
                )?);

                // Always some
                let context = self.modrinth.as_mut();
                unsafe { context.unwrap_unchecked() }
            }
        };

        Ok(context)
    }
}

impl<'a> PackCompiler<'a> {
    pub async fn deploy(&self, api_context: &Mutex<DeployAPIContext>) -> anyhow::Result<()> {
        if let Ok(context) = &mut api_context.lock() {
            let modrinth_context = context.init_modrinth()?;
            let mut path = self.compile_path.to_string_lossy().to_string();
            path += ".zip";
            println!("{}", path);
            let file_contents = std::fs::read(path)?;
            let response = modrinth_context
                .create_version(&self.as_modrinth_version(), "API_TEST.zip", file_contents)
                .await?;

            println!("{:#?}", response);

            Ok(())
        } else {
            Err(anyhow!("Failed to secure api context."))
        }
    }

    pub fn as_modrinth_version(&self) -> CreateVersion {
        CreateVersion {
            name: "API TEST".to_string(),
            version_number: "test2".to_string(),
            changelog: Some("# **this is an API test**".to_string()),
            dependencies: vec![],
            game_versions: vec!["1.9".to_string()],
            version_type: VersionType::Alpha,
            loaders: vec!["minecraft".to_string()],
            featured: false,
            status: Some(ferinth::structures::version::Status::Listed),
            requested_status: Some(ferinth::structures::version::RequestedStatus::Listed),
            project_id: "FRSckbRo".to_string(),
            file_parts: vec!["file".to_string()],
            primary_file: None,
        }
    }
}
