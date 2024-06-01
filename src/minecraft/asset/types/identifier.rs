use std::{
    convert::Infallible,
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Context;
use serde::{de::Visitor, Deserialize, Serialize};

const DEFAULT_NAMESPACE: &str = "minecraft";

#[derive(Debug, PartialEq, Eq)]
pub enum AssetType {
    Model,
    Blockstate,
}

#[derive(Debug, PartialEq, Eq, Hash)]
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

    pub fn from_path(value: &Path) -> anyhow::Result<(AssetType, Self)> {
        let mut path_list = value.iter();

        let namespace = path_list
            .next()
            .map_or(None, |f| f.to_str())
            .with_context(|| {
                format!(
                    "Failed to parse namespace from path: {}",
                    value.to_string_lossy()
                )
            })?;

        let asset_type = path_list
            .next()
            .map_or(None, |f| f.to_str())
            .map_or(None, |f| match f {
                "models" => Some(AssetType::Model),
                "blockstates" => Some(AssetType::Blockstate),
                _ => None,
            })
            .with_context(|| {
                format!(
                    "Failed to parse asset type from path: {}",
                    value.to_string_lossy()
                )
            })?;

        let asset_path = path_list.collect::<PathBuf>().with_extension("");

        Ok((asset_type, Identifier::new(namespace, asset_path)))
    }
}

impl FromStr for Identifier {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (namespace, path) = s.split_once(":").unwrap_or_else(|| (DEFAULT_NAMESPACE, s));
        Ok(Self::new(namespace, PathBuf::from(path)))
    }
}

impl From<Identifier> for String {
    fn from(value: Identifier) -> Self {
        value.to_string()
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!(
            "{}:{}",
            self.namespace,
            self.path.to_string_lossy()
        ))
    }
}

impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(IdentifierVisitor)
    }
}

struct IdentifierVisitor;

impl<'de> Visitor<'de> for IdentifierVisitor {
    type Value = Identifier;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a minecraft resource identifier; optionally, with a namespace")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Identifier::from_str(v).map_err(|e| E::custom(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_to_identifier_namespace_minecraft() {
        let raw = "minecraft:block/oak_planks";
        let expected = Identifier::minecraft("block/oak_planks");
        assert_eq!(expected, Identifier::from_str(raw).unwrap());
    }

    #[test]
    fn string_to_identifier_namespace_other() {
        let raw = "quark:block/oak_planks";
        let expected = Identifier::new("quark", "block/oak_planks");
        assert_eq!(expected, Identifier::from_str(raw).unwrap());
    }

    #[test]
    fn string_to_identifier() {
        let raw = "block/oak_planks";
        let expected = Identifier::minecraft("block/oak_planks");
        assert_eq!(expected, Identifier::from_str(raw).unwrap());
    }

    #[test]
    fn identifier_to_string_minecraft() {
        let identifier = Identifier::minecraft("block/dirt");
        assert_eq!("minecraft:block/dirt", String::from(identifier));
    }

    #[test]
    fn identifier_to_string_other() {
        let id = Identifier::new("quark", "block/dirt");
        assert_eq!("quark:block/dirt", String::from(id));
    }

    #[test]
    fn from_path_minecraft_model() {
        let id = Identifier::minecraft("block/sponge");
        let result =
            Identifier::from_path(Path::new("minecraft/models/block/sponge.json")).unwrap();
        assert_eq!((AssetType::Model, id), result);
    }

    #[test]
    fn from_path_minecraft_blockstate() {
        let id = Identifier::minecraft("block/sponge");
        let result =
            Identifier::from_path(Path::new("minecraft/blockstates/block/sponge.json")).unwrap();
        assert_eq!((AssetType::Blockstate, id), result);
    }

    #[test]
    fn from_path_other() {
        let id = Identifier::new("quark", "block/sponge");
        let result = Identifier::from_path(Path::new("quark/models/block/sponge.json")).unwrap();
        assert_eq!((AssetType::Model, id), result);
    }
}
