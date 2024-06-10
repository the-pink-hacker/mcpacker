pub mod formatting;

use std::{fmt::Display, ops::AddAssign};

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use self::formatting::FormattingCode;

use super::identifier::Identifier;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RawText {
    Raw(String),
    List(Vec<RawText>),
    Single(TextComponent),
}

impl RawText {
    pub fn len(&self) -> usize {
        match self {
            Self::Raw(text) => text.len(),
            Self::List(texts) => texts.iter().map(Self::len).sum(),
            Self::Single(text) => text.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Raw(text) => text.is_empty(),
            Self::List(texts) => {
                for text in texts {
                    if !text.is_empty() {
                        return false;
                    }
                }
                true
            }
            Self::Single(text) => text.is_empty(),
        }
    }
}

impl From<String> for RawText {
    fn from(value: String) -> Self {
        Self::Raw(value)
    }
}

impl From<&str> for RawText {
    fn from(value: &str) -> Self {
        Self::Raw(value.to_string())
    }
}

impl AddAssign for RawText {
    fn add_assign(&mut self, rhs: Self) {
        match self {
            Self::Raw(_) | Self::Single(_) => *self = Self::List(vec![self.clone(), rhs]),
            Self::List(list) => list.push(rhs),
        }
    }
}

impl Default for RawText {
    fn default() -> Self {
        Self::Raw(Default::default())
    }
}

impl Display for RawText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Raw(text) => f.write_str(text),
            Self::List(texts) => {
                f.write_str(&texts.iter().map(RawText::to_string).collect::<String>())
            }
            Self::Single(text) => f.write_str(&text.to_string()),
        }
    }
}

// TODO: Add interactivity fields.
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    extra: Vec<RawText>,
}

impl TextComponent {
    pub fn len(&self) -> usize {
        let extra_length = self.extra.iter().map(RawText::len).sum::<usize>();

        extra_length
            + match &self.text {
                TextType::Text { text } => text.len(),
                _ => 0,
            }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
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

        for text in &self.extra {
            output += &text.to_string();
        }

        f.write_str(&output)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextType {
    Text { text: String },
    Translatable,
    Score,
    Selector,
    Keybind,
    NBT,
}

impl From<String> for TextType {
    fn from(value: String) -> Self {
        Self::Text { text: value }
    }
}

impl From<&str> for TextType {
    fn from(value: &str) -> Self {
        Self::Text {
            text: value.to_string(),
        }
    }
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
        let raw = RawText::from("test");
        assert_eq!(expected, raw.to_string());
    }

    #[test]
    fn raw_to_string_color() {
        let expected = "\u{A7}ctest";
        let raw = RawText::Single(TextComponent {
            text: "test".into(),
            color: Some(TextColor::Red),
            ..Default::default()
        });
        assert_eq!(expected, raw.to_string());
    }

    #[test]
    fn raw_to_string_formatting() {
        let expected = "\u{A7}f\u{A7}k\u{A7}l\u{A7}m\u{A7}n\u{A7}otest";
        let raw = RawText::Single(TextComponent {
            text: "test".into(),
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
            text: "one".into(),
            extra: vec![" two".into(), " three".into()],
            ..Default::default()
        });
        assert_eq!(expected, raw.to_string());
    }

    #[test]
    fn raw_to_string_list() {
        let expected = "one two three";
        let raw = RawText::List(vec!["one".into(), " two".into(), " three".into()]);
        assert_eq!(expected, raw.to_string());
    }

    #[test]
    fn length_raw() {
        let raw = RawText::from("abc");
        assert_eq!(3, raw.len());
    }

    #[test]
    fn length_list() {
        let raw = RawText::List(vec!["a".into(), "b".into(), "cd".into()]);
        assert_eq!(4, raw.len());
    }

    #[test]
    fn length_single() {
        let raw = RawText::Single(TextComponent {
            text: "words".into(),
            ..Default::default()
        });
        assert_eq!(5, raw.len());
    }

    #[test]
    fn length_extra() {
        let raw = RawText::Single(TextComponent {
            text: "a".into(),
            extra: vec!["b".into(), "c".into()],
            ..Default::default()
        });
        assert_eq!(3, raw.len());
    }

    #[test]
    fn is_empty_raw() {
        let raw = RawText::from("text");
        assert!(!raw.is_empty());
    }

    #[test]
    fn is_empty_raw_empty() {
        let raw = RawText::from("");
        assert!(raw.is_empty());
    }

    #[test]
    fn is_empty_list() {
        let raw = RawText::List(vec!["".into(), "hi there".into()]);
        assert!(!raw.is_empty());
    }

    #[test]
    fn is_empty_list_empty() {
        let raw = RawText::List(vec!["".into()]);
        assert!(raw.is_empty());
    }

    #[test]
    fn is_empty_single() {
        let raw = RawText::Single(TextComponent {
            text: "words".into(),
            ..Default::default()
        });
        assert!(!raw.is_empty());
    }

    #[test]
    fn is_empty_single_empty() {
        let raw = RawText::Single(TextComponent {
            text: "".into(),
            ..Default::default()
        });
        assert!(raw.is_empty());
    }

    #[test]
    fn is_empty_extra() {
        let raw = RawText::Single(TextComponent {
            extra: vec!["".into(), "hello".into()],
            ..Default::default()
        });
        assert!(!raw.is_empty());
    }

    #[test]
    fn is_empty_extra_empty() {
        let raw = RawText::Single(TextComponent {
            extra: vec!["".into()],
            ..Default::default()
        });
        assert!(raw.is_empty());
    }
}
