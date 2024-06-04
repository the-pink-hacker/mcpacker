pub mod formatting;

use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::identifier::Identifier;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RawText {
    Raw(String),
    List(Vec<RawText>),
    Single(TextComponent),
}

impl Display for RawText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Self::Raw(text) => text,
            Self::List(_) => unimplemented!(),
            Self::Single(_) => unimplemented!(),
        };

        f.write_str(output)
    }
}

// TODO: Add interactivity fields.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct TextComponent {
    #[serde(flatten)]
    text: TextType,
    color: Option<TextColor>,
    font: Option<Identifier>,
    bold: Option<bool>,
    italics: Option<bool>,
    underline: Option<bool>,
    strikethrough: Option<bool>,
    obfuscated: Option<bool>,
    extra: Option<Vec<RawText>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextType {
    Text,
    Translatable,
    Score,
    Selector,
    Keybind,
    NBT,
}

// TODO: Add hex color codes.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextColor {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_to_string() {
        let expected = "test";
        let raw = RawText::Raw("test".to_string());
        assert_eq!(expected, raw.to_string());
    }
}
