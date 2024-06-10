use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use path_clean::PathClean;

pub struct PathSanitizer {
    pub restricted_path: PathBuf,
}

impl PathSanitizer {
    pub fn join<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<PathBuf> {
        self.sanitize(self.restricted_path.join(path))
    }

    pub fn sanitize<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<PathBuf> {
        let path = path.as_ref().clean();

        if path.strip_prefix(&self.restricted_path).is_err() {
            Err(anyhow!(
                "Illegal path: {}; restricted to: {}",
                path.display(),
                self.restricted_path.display()
            ))
        } else {
            Ok(path)
        }
    }
}

impl TryFrom<PathBuf> for PathSanitizer {
    type Error = anyhow::Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        Ok(Self {
            restricted_path: value
                .canonicalize()
                .with_context(|| format!("Failed to get canon path: {}", value.display()))?,
        })
    }
}

impl TryFrom<&Path> for PathSanitizer {
    type Error = anyhow::Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        Ok(Self {
            restricted_path: value
                .canonicalize()
                .with_context(|| format!("Failed to get canon path: {}", value.display()))?,
        })
    }
}
