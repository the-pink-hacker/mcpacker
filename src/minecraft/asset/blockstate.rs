use super::types::{identifier::Identifier, rotation::StateRotation};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Blockstate {
    Variants(IndexMap<String, ModelState>),
    Multipart(Vec<BlockstateMultipart>),
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
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

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct WeightedState {
    model: Identifier,
    x: Option<StateRotation>,
    y: Option<StateRotation>,
    uvlock: Option<bool>,
    weight: Option<u8>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct BlockstateMultipart {
    when: Option<MultipartCondition>,
    apply: ModelState,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MultipartCondition {
    And {
        #[serde(rename = "AND")]
        and: Vec<IndexMap<String, StateValue>>,
    },
    Or {
        #[serde(rename = "OR")]
        or: Vec<IndexMap<String, StateValue>>,
    },
    Single(IndexMap<String, StateValue>),
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StateValue {
    Boolean(bool),
    Integer(u8),
    Enum(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! deserialize_test {
        ($name:ident, $raw:literal, $expected: expr $(,)?) => {
            #[test]
            fn $name() {
                let parsed: Blockstate = serde_json::from_str($raw).unwrap();
                assert_eq!(parsed, $expected);
            }
        };
    }

    deserialize_test!(
        deserialize_variant_single,
        r#"{
            "variants": {
                "test=1234": {
                    "model": "minecraft:block/dirt",
                    "x": 180,
                    "uvlock": true
                }
            }
        }"#,
        Blockstate::Variants(IndexMap::from([(
            "test=1234".to_string(),
            ModelState::Single {
                model: Identifier::minecraft("block/dirt"),
                x: Some(StateRotation::Degrees180),
                y: None,
                uvlock: Some(true),
            },
        )]))
    );

    deserialize_test!(
        deserialize_variant_multiple,
        r#"{
            "variants": {
                "": [
                    {
                        "model": "minecraft:block/grass_block"
                    },
                    {
                        "model": "minecraft:block/grass_block",
                        "x": 450,
                        "weight": 2
                    }
                ]
            }
        }"#,
        Blockstate::Variants(IndexMap::from([(
            String::new(),
            ModelState::Weighted(vec![
                WeightedState {
                    model: Identifier::minecraft("block/grass_block"),
                    x: None,
                    y: None,
                    uvlock: None,
                    weight: None,
                },
                WeightedState {
                    model: Identifier::minecraft("block/grass_block"),
                    x: Some(StateRotation::Degrees90),
                    y: None,
                    uvlock: None,
                    weight: Some(2),
                }
            ])
        )]))
    );
}
