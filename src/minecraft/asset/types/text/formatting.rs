use std::fmt::Display;

use super::TextColor;

pub enum FormattingCode {
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
    Obfuscated,
    Bold,
    Strikethrough,
    Underline,
    Italic,
    Reset,
}

macro_rules! code {
    ($value: literal) => {
        concat!("\u{A7}", $value)
    };
}

impl Display for FormattingCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Self::Black => code!('0'),
            Self::DarkBlue => code!('1'),
            Self::DarkGreen => code!('2'),
            Self::DarkAqua => code!('3'),
            Self::DarkRed => code!('4'),
            Self::DarkPurple => code!('5'),
            Self::Gold => code!('6'),
            Self::Gray => code!('7'),
            Self::DarkGray => code!('8'),
            Self::Blue => code!('9'),
            Self::Green => code!('a'),
            Self::Aqua => code!('b'),
            Self::Red => code!('c'),
            Self::LightPurple => code!('d'),
            Self::Yellow => code!('e'),
            Self::White => code!('f'),
            Self::Obfuscated => code!('k'),
            Self::Bold => code!('l'),
            Self::Strikethrough => code!('m'),
            Self::Underline => code!('n'),
            Self::Italic => code!('o'),
            Self::Reset => code!('r'),
        };

        f.write_str(output)
    }
}

impl From<TextColor> for FormattingCode {
    fn from(value: TextColor) -> Self {
        match value {
            TextColor::Black => Self::Black,
            TextColor::DarkBlue => Self::DarkBlue,
            TextColor::DarkGreen => Self::DarkGreen,
            TextColor::DarkAqua => Self::DarkAqua,
            TextColor::DarkRed => Self::DarkRed,
            TextColor::DarkPurple => Self::DarkPurple,
            TextColor::Gold => Self::Gold,
            TextColor::Gray => Self::Gray,
            TextColor::DarkGray => Self::DarkGray,
            TextColor::Blue => Self::Blue,
            TextColor::Green => Self::Green,
            TextColor::Aqua => Self::Aqua,
            TextColor::Red => Self::Red,
            TextColor::LightPurple => Self::LightPurple,
            TextColor::Yellow => Self::Yellow,
            TextColor::White => Self::White,
        }
    }
}
