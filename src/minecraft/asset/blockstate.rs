use crate::minecraft::serialize::*;

use super::{
    types::{
        identifier::{AssetType, Identifier},
        rotation::StateRotation,
    },
    Asset,
};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Blockstate {
    Variants(IndexMap<String, ModelState>),
    Multipart(Vec<BlockstateMultipart>),
}

impl Asset for Blockstate {
    fn get_type() -> AssetType {
        AssetType::Blockstate
    }
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ModelState {
    Single {
        model: Identifier,
        #[serde(default, skip_serializing_if = "StateRotation::is_default")]
        x: StateRotation,
        #[serde(default, skip_serializing_if = "StateRotation::is_default")]
        y: StateRotation,
        #[serde(default, skip_serializing_if = "is_false")]
        uvlock: bool,
    },
    Weighted(Vec<WeightedState>),
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct WeightedState {
    model: Identifier,
    #[serde(default, skip_serializing_if = "StateRotation::is_default")]
    x: StateRotation,
    #[serde(default, skip_serializing_if = "StateRotation::is_default")]
    y: StateRotation,
    #[serde(default, skip_serializing_if = "is_false")]
    uvlock: bool,
    #[serde(
        default = "WeightedState::default_weight",
        skip_serializing_if = "WeightedState::is_weight_default"
    )]
    weight: u8,
}

impl WeightedState {
    #[inline]
    fn default_weight() -> u8 {
        1
    }

    #[inline]
    fn is_weight_default(value: &u8) -> bool {
        *value == Self::default_weight()
    }
}

#[skip_serializing_none]
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
                x: StateRotation::Degrees180,
                y: StateRotation::Degrees0,
                uvlock: true,
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
                    x: StateRotation::Degrees0,
                    y: StateRotation::Degrees0,
                    uvlock: false,
                    weight: 1,
                },
                WeightedState {
                    model: Identifier::minecraft("block/grass_block"),
                    x: StateRotation::Degrees90,
                    y: StateRotation::Degrees0,
                    uvlock: false,
                    weight: 2,
                }
            ])
        )]))
    );
}
