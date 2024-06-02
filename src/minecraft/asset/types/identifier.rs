use std::{
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{bail, Context};
use serde::{de::Visitor, Deserialize, Serialize};

const DEFAULT_NAMESPACE: &str = "minecraft";

#[derive(Debug, PartialEq, Eq)]
pub enum AssetType {
    Model,
    Blockstate,
    Texture,
    Atlas,
    TextureMeta,
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
                "textures" => value.extension().map_or(None, |e| {
                    match e.to_string_lossy().to_mut().clone().as_str() {
                        "png" => Some(AssetType::Texture),
                        "mcmeta" => Some(AssetType::TextureMeta),
                        _ => None,
                    }
                }),
                "atlases" => Some(AssetType::Atlas),
                _ => None,
            })
            .with_context(|| {
                format!(
                    "Failed to parse asset type from path: {}",
                    value.to_string_lossy()
                )
            })?;

        let asset_path = path_list
            .collect::<PathBuf>()
            .with_extension("")
            .with_extension("");

        Ok((asset_type, Identifier::new(namespace, asset_path)))
    }

    pub fn to_path(&self, asset_path: &PathBuf, asset_type: &AssetType) -> PathBuf {
        let (folder, extension) = match asset_type {
            AssetType::Model => ("models", "json"),
            AssetType::Blockstate => ("blockstates", "json"),
            AssetType::Texture => ("textures", "png"),
            AssetType::Atlas => ("atlases", "json"),
            AssetType::TextureMeta => ("textures", "png.mcmeta"),
        };

        asset_path
            .join(&self.namespace)
            .join(folder)
            .join(&self.path)
            .with_extension(extension)
    }
}

impl FromStr for Identifier {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('#') {
            bail!("Can't parse identifier; variable detected.");
        }

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
    fn from_path_minecraft_texture() {
        let id = Identifier::minecraft("block/crafting_table_top");
        let result =
            Identifier::from_path(Path::new("minecraft/textures/block/crafting_table_top.png"))
                .unwrap();
        assert_eq!((AssetType::Texture, id), result);
    }

    #[test]
    fn from_path_minecraft_texture_meta() {
        let id = Identifier::minecraft("block/crafting_table_top");
        let result = Identifier::from_path(Path::new(
            "minecraft/textures/block/crafting_table_top.png.mcmeta",
        ))
        .unwrap();
        assert_eq!((AssetType::TextureMeta, id), result);
    }

    #[test]
    fn from_path_minecraft_atlas() {
        let id = Identifier::minecraft("blocks");
        let result = Identifier::from_path(Path::new("minecraft/atlases/blocks.json")).unwrap();
        assert_eq!((AssetType::Atlas, id), result);
    }

    #[test]
    fn from_path_other() {
        let id = Identifier::new("quark", "block/sponge");
        let result = Identifier::from_path(Path::new("quark/models/block/sponge.json")).unwrap();
        assert_eq!((AssetType::Model, id), result);
    }
}
