use serde::{Deserialize, Serialize};
use serde_json::ser::PrettyFormatter;

use super::{FormatType, PackMetaConfig};

#[derive(Debug, Serialize, Default)]
pub struct PackMCMetaContents {
    pack_format: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    supported_formats: Option<FormatType>,
}

#[derive(Debug, Serialize, Default)]
pub struct PackMCMeta {
    pub pack: PackMCMetaContents,
}

impl From<&PackMetaConfig> for PackMCMeta {
    fn from(value: &PackMetaConfig) -> Self {
        let (pack_format, supported_formats) =
            value.format.clone().unwrap_or_default().get_formats();

        Self {
            pack: PackMCMetaContents {
                description: value.description.clone(),
                pack_format,
                supported_formats,
            },
        }
    }
}

#[derive(Debug, Deserialize, Default, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExportOutputType {
    #[default]
    Uncompressed,
    Zip,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ExportRelocation {
    #[default]
    None,
    Copy,
    Symbolic,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "snake_case")]
pub enum JsonExportType {
    #[default]
    Compact,
    Pretty,
}

impl JsonExportType {
    pub fn to_string<T>(&self, value: &T) -> anyhow::Result<String>
    where
        T: ?Sized + Serialize,
    {
        match self {
            Self::Compact => Ok(serde_json::to_string(value)?),
            Self::Pretty => {
                let formatter = PrettyFormatter::with_indent(b"    ");
                let mut writter = Vec::new();
                let mut serializer =
                    serde_json::ser::Serializer::with_formatter(&mut writter, formatter);

                value.serialize(&mut serializer)?;
                Ok(String::from_utf8(writter)?)
            }
        }
    }
}
