use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const DEFAULT_NAMESPACE: &str = "minecraft";

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Identifier {
    namespace: String,
    path: PathBuf,
}

impl Identifier {
    pub fn new(namespace: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }

    pub fn minecraft(path: impl Into<PathBuf>) -> Self {
        Self {
            namespace: DEFAULT_NAMESPACE.into(),
            path: path.into(),
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.namespace, self.path.to_string_lossy())
    }
}

impl TryFrom<&str> for Identifier {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (namespace, path) = value
            .split_once(':')
            .unwrap_or_else(|| (DEFAULT_NAMESPACE, value));
        Ok(Self::new(namespace, PathBuf::try_from(path)?))
    }
}

impl TryFrom<String> for Identifier {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let (namespace, path) = value
            .split_once(':')
            .unwrap_or_else(|| (DEFAULT_NAMESPACE, &value));
        Ok(Self::new(namespace, PathBuf::try_from(path)?))
    }
}

impl From<Identifier> for String {
    fn from(value: Identifier) -> Self {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_to_identifier_namespace_minecraft() {
        let raw = "minecraft:block/oak_planks";
        let expected = Identifier::minecraft("block/oak_planks");
        assert_eq!(expected, Identifier::try_from(raw).unwrap());
    }

    #[test]
    fn string_to_identifier_namespace_other() {
        let raw = "quark:block/oak_planks";
        let expected = Identifier::new("quark", "block/oak_planks");
        assert_eq!(expected, Identifier::try_from(raw).unwrap());
    }

    #[test]
    fn string_to_identifier() {
        let raw = "block/oak_planks";
        let expected = Identifier::minecraft("block/oak_planks");
        assert_eq!(expected, Identifier::try_from(raw).unwrap());
    }

    #[test]
    fn identifier_to_string_minecraft() {
        let identifier = Identifier::minecraft("block/dirt");
        assert_eq!("minecraft:block/dirt", Into::<String>::into(identifier));
    }

    #[test]
    fn identifier_to_string_other() {
        let identifier = Identifier::new("quark", "block/dirt");
        assert_eq!("quark:block/dirt", Into::<String>::into(identifier));
    }
}
