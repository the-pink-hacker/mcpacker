use anyhow::anyhow;
use tempfile::NamedTempFile;

pub struct Changelog {
    temp_file: NamedTempFile,
}

impl Changelog {
    pub fn new() -> anyhow::Result<Self> {
        let temp_file = tempfile::NamedTempFile::new()?;

        Ok(Self { temp_file })
    }

    pub async fn collect_changelog(self) -> anyhow::Result<String> {
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

        async_process::Command::new(editor)
            .arg(self.temp_file.path())
            .status()
            .await?;

        let changelog = async_fs::read_to_string(self.temp_file).await?;

        if changelog.is_empty() {
            Err(anyhow!("Changelog was unchanged."))
        } else {
            Ok(changelog)
        }
    }
}
