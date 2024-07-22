use std::{fmt::Display, str::FromStr};

use anyhow::Context;
use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(Debug, Clone, PartialEq, Eq, DeserializeFromStr, SerializeDisplay)]
pub struct VariableIdentifier(String);

impl VariableIdentifier {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn get_name(&self) -> &str {
        &self.0
    }
}

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
