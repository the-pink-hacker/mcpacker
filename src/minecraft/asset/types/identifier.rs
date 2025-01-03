use std::{
    ffi::OsStr,
    fmt,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, bail, Context};
use serde::{de::Visitor, Deserialize, Serialize};

const DEFAULT_NAMESPACE: &str = "minecraft";

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Model,
    Blockstate,
    Texture,
    Atlas,
    TextureMeta,
    Sound,
    Particle,
    Text,
    Language,
    Modifier,
    ModifierIndex,
    ItemModelDefinition,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: PathBuf,
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

    pub fn from_path<P: AsRef<Path>>(value: P) -> anyhow::Result<(AssetType, Self)> {
        let value = value.as_ref();
        let mut path_list = value.iter();

        let namespace = path_list.next().and_then(|f| f.to_str()).with_context(|| {
            format!(
                "Failed to parse namespace from path: {}",
                value.to_string_lossy()
            )
        })?;

        let asset_type = path_list
            .next()
            .and_then(OsStr::to_str)
            .with_context(|| {
                format!(
                    "Failed to parse top level path: {}",
                    value.to_string_lossy()
                )
            })
            .and_then(|f| match f {
                "models" => value
                    .iter()
                    .nth(2)
                    .with_context(|| {
                        format!("Top level models not allow: {}", value.to_string_lossy())
                    })?
                    .to_str()
                    .with_context(|| {
                        format!("Failed to parse model folder: {}", value.to_string_lossy())
                    })
                    .and_then(|folder| match folder {
                        "block" | "item" => Ok(AssetType::Model),
                        "template" => {
                            eprintln!(
                                "[WARNING] Top level templates deprecated: {}",
                                value.to_string_lossy()
                            );
                            Ok(AssetType::Model)
                        }
                        _ => Err(anyhow!(
                            "Unsupported model type '{}': {}",
                            folder,
                            value.to_string_lossy()
                        )),
                    }),
                "blockstates" => Ok(AssetType::Blockstate),
                "textures" => value
                    .extension()
                    .and_then(OsStr::to_str)
                    .with_context(|| {
                        format!(
                            "Failed to parse texture extension: {}",
                            value.to_string_lossy()
                        )
                    })
                    .and_then(|extension| match extension {
                        "png" => Ok(AssetType::Texture),
                        "mcmeta" => Ok(AssetType::TextureMeta),
                        _ => Err(anyhow!(
                            "Texture extension '{}' unsupported: {}",
                            extension,
                            value.to_string_lossy()
                        )),
                    }),
                "atlases" => Ok(AssetType::Atlas),
                "modifiers" => Ok(AssetType::Modifier),
                "modifiers.toml" => Ok(AssetType::ModifierIndex),
                "items" => Ok(AssetType::ItemModelDefinition),
                _ => Err(anyhow!(
                    "Unsupported asset type '{}': {}",
                    f,
                    value.to_string_lossy()
                )),
            })?;

        let asset_path = path_list
            .collect::<PathBuf>()
            .with_extension("")
            .with_extension("");

        Ok((asset_type, Identifier::new(namespace, asset_path)))
    }

    pub fn to_path(&self, asset_path: &Path, asset_type: &AssetType) -> PathBuf {
        let (folder, extension) = match asset_type {
            AssetType::Model => ("models", "json"),
            AssetType::Blockstate => ("blockstates", "json"),
            AssetType::Texture => ("textures", "png"),
            AssetType::Atlas => ("atlases", "json"),
            AssetType::TextureMeta => ("textures", "png.mcmeta"),
            AssetType::Sound => ("sounds", "ogg"),
            AssetType::Text => ("texts", "txt"),
            AssetType::Particle => ("particles", "json"),
            AssetType::Language => ("lang", "json"),
            AssetType::Modifier => ("modifiers", "toml"),
            AssetType::ModifierIndex => (".", "toml"),
            AssetType::ItemModelDefinition => ("items", "json"),
        };

        asset_path
            .join(&self.namespace)
            .join(folder)
            .join(&self.path)
            .with_extension(extension)
    }

    pub fn is_minecraft(&self) -> bool {
        self.namespace == DEFAULT_NAMESPACE
    }

    pub fn starts_with(&self, other: &Identifier) -> bool {
        self.namespace == other.namespace && self.path.starts_with(&other.path)
    }
}

impl FromStr for Identifier {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('#') {
            bail!("Can't parse identifier; variable detected.");
        }

        let (namespace, path) = s.split_once(':').unwrap_or((DEFAULT_NAMESPACE, s));
        Ok(Identifier::new(namespace, path))
    }
}

impl From<Identifier> for String {
    fn from(value: Identifier) -> Self {
        value.to_string()
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = if self.is_minecraft() {
            self.path.to_string_lossy().to_string()
        } else {
            format!("{}:{}", self.namespace, self.path.to_string_lossy())
        };
        f.write_str(&output)
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

impl Visitor<'_> for IdentifierVisitor {
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
        assert_eq!("block/dirt", String::from(identifier));
    }

    #[test]
    fn identifier_to_string_other() {
        let id = Identifier::new("quark", "block/dirt");
        assert_eq!("quark:block/dirt", String::from(id));
    }

    #[test]
    fn from_path_minecraft_model_block() {
        let id = Identifier::minecraft("block/sponge");
        let result = Identifier::from_path("minecraft/models/block/sponge.json").unwrap();
        assert_eq!((AssetType::Model, id), result);
    }

    #[test]
    fn from_path_minecraft_model_item() {
        let id = Identifier::minecraft("item/sponge");
        let result = Identifier::from_path("minecraft/models/item/sponge.json").unwrap();
        assert_eq!((AssetType::Model, id), result);
    }

    #[test]
    fn from_path_minecraft_model_invalid() {
        let result = Identifier::from_path("minecraft/models/sponge.json");
        assert!(result.is_err());
    }

    #[test]
    fn from_path_minecraft_item_definition() {
        let id = Identifier::minecraft("sponge");
        let result = Identifier::from_path("minecraft/items/sponge.json").unwrap();
        assert_eq!((AssetType::ItemModelDefinition, id), result);
    }

    #[test]
    fn from_path_minecraft_blockstate() {
        let id = Identifier::minecraft("block/sponge");
        let result = Identifier::from_path("minecraft/blockstates/block/sponge.json").unwrap();
        assert_eq!((AssetType::Blockstate, id), result);
    }

    #[test]
    fn from_path_minecraft_texture() {
        let id = Identifier::minecraft("block/crafting_table_top");
        let result =
            Identifier::from_path("minecraft/textures/block/crafting_table_top.png").unwrap();
        assert_eq!((AssetType::Texture, id), result);
    }

    #[test]
    fn from_path_minecraft_texture_meta() {
        let id = Identifier::minecraft("block/crafting_table_top");
        let result =
            Identifier::from_path("minecraft/textures/block/crafting_table_top.png.mcmeta")
                .unwrap();
        assert_eq!((AssetType::TextureMeta, id), result);
    }

    #[test]
    fn from_path_minecraft_atlas() {
        let id = Identifier::minecraft("blocks");
        let result = Identifier::from_path("minecraft/atlases/blocks.json").unwrap();
        assert_eq!((AssetType::Atlas, id), result);
    }

    #[test]
    fn from_path_other() {
        let id = Identifier::new("quark", "block/sponge");
        let result = Identifier::from_path("quark/models/block/sponge.json").unwrap();
        assert_eq!((AssetType::Model, id), result);
    }

    #[test]
    fn starts_with_match() {
        let id = Identifier::minecraft("block/dirt");
        let other = Identifier::minecraft("block");
        assert!(id.starts_with(&other));
    }

    #[test]
    fn starts_with_namespace_miss() {
        let id = Identifier::minecraft("block/dirt");
        let other = Identifier::new("quark", "block");
        assert!(!id.starts_with(&other));
    }

    #[test]
    fn starts_with_miss() {
        let id = Identifier::minecraft("block/dirt");
        let other = Identifier::minecraft("item");
        assert!(!id.starts_with(&other));
    }
}
