use std::{fmt::Display, str::FromStr};

use anyhow::Context;
use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(Debug, DeserializeFromStr, SerializeDisplay)]
pub struct VariableIdentifier(String);

impl Display for VariableIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::from('#');
        output += &self.0;

        f.write_str(&output)
    }
}

impl FromStr for VariableIdentifier {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let variable = s
            .strip_prefix('#')
            .with_context(|| format!("Failed to parse identifier variable: {}", s))?
            .to_string();
        Ok(Self(variable))
    }
}
