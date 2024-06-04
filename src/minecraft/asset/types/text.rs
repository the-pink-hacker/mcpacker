pub mod formatting;

use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use self::formatting::FormattingCode;

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
        match self {
            Self::Raw(text) => f.write_str(text),
            Self::List(_) => unimplemented!(),
            Self::Single(text) => f.write_str(&text.to_string()),
        }
    }
}

// TODO: Add interactivity fields.
#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize)]
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

impl Display for TextComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        if let Some(color) = &self.color {
            output += &FormattingCode::from(color.clone()).to_string();
        }

        if self.obfuscated == Some(true) {
            output += &FormattingCode::Obfuscated.to_string();
        }

        if self.bold == Some(true) {
            output += &FormattingCode::Bold.to_string();
        }

        if self.strikethrough == Some(true) {
            output += &FormattingCode::Strikethrough.to_string();
        }

        if self.underline == Some(true) {
            output += &FormattingCode::Underline.to_string();
        }

        if self.italics == Some(true) {
            output += &FormattingCode::Italic.to_string();
        }

        output += match &self.text {
            TextType::Text { text } => text,
            _ => unimplemented!(),
        };

        if let Some(extra) = &self.extra {
            for text in extra {
                output += &text.to_string();
            }
        }

        f.write_str(&output)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextType {
    Text { text: String },
    Translatable,
    Score,
    Selector,
    Keybind,
    NBT,
}

impl Default for TextType {
    fn default() -> Self {
        Self::Text {
            text: Default::default(),
        }
    }
}

// TODO: Add hex color codes.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    #[test]
    fn raw_to_string_color() {
        let expected = "\u{A7}ctest";
        let raw = RawText::Single(TextComponent {
            text: TextType::Text {
                text: "test".to_string(),
            },
            color: Some(TextColor::Red),
            ..Default::default()
        });
        assert_eq!(expected, raw.to_string());
    }

    #[test]
    fn raw_to_string_formatting() {
        let expected = "\u{A7}f\u{A7}k\u{A7}l\u{A7}m\u{A7}n\u{A7}otest";
        let raw = RawText::Single(TextComponent {
            text: TextType::Text {
                text: "test".to_string(),
            },
            color: Some(TextColor::White),
            italics: Some(true),
            obfuscated: Some(true),
            bold: Some(true),
            strikethrough: Some(true),
            underline: Some(true),
            ..Default::default()
        });
        assert_eq!(expected, raw.to_string());
    }

    #[test]
    fn raw_to_string_extra() {
        let expected = "one two three";
        let raw = RawText::Single(TextComponent {
            text: TextType::Text {
                text: "one".to_string(),
            },
            extra: Some(vec![
                RawText::Raw(" two".to_string()),
                RawText::Raw(" three".to_string()),
            ]),
            ..Default::default()
        });
        assert_eq!(expected, raw.to_string());
    }
}
