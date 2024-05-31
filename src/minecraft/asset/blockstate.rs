use std::collections::HashMap;

use super::identifier::Identifier;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Blockstate {
    Varient {
        variants: HashMap<String, ModelState>,
    },
    Multipart {
        multipart: Vec<BlockstateMultipart>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ModelState {
    Single {
        model: Identifier,
        x: Option<StateRotation>,
        y: Option<StateRotation>,
        uvlock: Option<bool>,
    },
    Weighted(Vec<WeightedState>),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WeightedState {
    model: Identifier,
    x: Option<StateRotation>,
    y: Option<StateRotation>,
    uvlock: Option<bool>,
    weight: Option<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum StateRotation {
    Degrees0,
    Degrees90,
    Degrees180,
    Degrees270,
    Degrees360,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct BlockstateMultipart {
    when: HashMap<String, StateValue>,
    apply: ModelState,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum StateValue {
    Boolean(bool),
    Integer(u8),
    Enum(String),
}
