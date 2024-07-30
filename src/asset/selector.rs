use serde::Deserialize;
use serde_with::{serde_as, OneOrMany};

use crate::minecraft::asset::types::identifier::Identifier;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AssetSelector {
    Whitelist { qualifier: AssetSelectorQualifier },
    Blacklist { qualifier: AssetSelectorQualifier },
}

impl Default for AssetSelector {
    fn default() -> Self {
        Self::Whitelist {
            qualifier: Default::default(),
        }
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AssetSelectorQualifier {
    #[default]
    All,
    Items {
        #[serde_as(as = "OneOrMany<_>")]
        items: Vec<Identifier>,
    },
    Paths {
        #[serde_as(as = "OneOrMany<_>")]
        items: Vec<Identifier>,
        #[serde(default)]
        namespace_specific: bool,
    },
}

impl AssetSelectorQualifier {
    pub fn applies(&self, id: &Identifier) -> bool {
        match self {
            Self::All => true,
            Self::Paths {
                items,
                namespace_specific,
            } => {
                if *namespace_specific {
                    items
                        .iter()
                        .any(|qualifier_path_id| id.starts_with(qualifier_path_id))
                } else {
                    items
                        .iter()
                        .any(|qualifier_path_id| id.path.starts_with(&qualifier_path_id.path))
                }
            }
            Self::Items { items } => items.contains(id),
        }
    }
}

impl AssetSelector {
    pub fn applies(&self, id: &Identifier) -> bool {
        match self {
            Self::Whitelist { qualifier } => qualifier.applies(id),
            Self::Blacklist { qualifier } => !qualifier.applies(id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitelist_all() {
        let selector = AssetSelector::Whitelist {
            qualifier: AssetSelectorQualifier::All,
        };
        let assets = vec![
            Identifier::minecraft("block/dirt"),
            Identifier::minecraft("item/grass_block"),
            Identifier::new("quark", "block/test"),
        ];

        for asset in &assets {
            assert!(selector.applies(asset));
        }
    }

    #[test]
    fn whitelist_paths() {
        let selector = AssetSelector::Whitelist {
            qualifier: AssetSelectorQualifier::Paths {
                items: vec![
                    Identifier::minecraft("block"),
                    Identifier::minecraft("item"),
                ],
                namespace_specific: false,
            },
        };

        assert!(selector.applies(&Identifier::minecraft("block/dirt")));
        assert!(selector.applies(&Identifier::minecraft("block/grass_block")));
        assert!(selector.applies(&Identifier::minecraft("item/oak_planks")));
        assert!(selector.applies(&Identifier::new("quark", "item/test")));
        assert!(!selector.applies(&Identifier::minecraft("entity/zombie")));
    }

    #[test]
    fn whitelist_paths_namespace_specific() {
        let selector = AssetSelector::Whitelist {
            qualifier: AssetSelectorQualifier::Paths {
                items: vec![
                    Identifier::minecraft("block"),
                    Identifier::minecraft("item"),
                ],
                namespace_specific: true,
            },
        };

        assert!(selector.applies(&Identifier::minecraft("block/dirt")));
        assert!(selector.applies(&Identifier::minecraft("block/grass_block")));
        assert!(selector.applies(&Identifier::minecraft("item/oak_planks")));
        assert!(!selector.applies(&Identifier::new("quark", "item/test")));
        assert!(!selector.applies(&Identifier::minecraft("entity/zombie")));
    }

    #[test]
    fn whitelist_items() {
        let selector = AssetSelector::Whitelist {
            qualifier: AssetSelectorQualifier::Items {
                items: vec![
                    Identifier::minecraft("block/dirt"),
                    Identifier::minecraft("block/grass_block"),
                    Identifier::minecraft("block/oak_planks"),
                ],
            },
        };

        assert!(selector.applies(&Identifier::minecraft("block/dirt")));
        assert!(selector.applies(&Identifier::minecraft("block/grass_block")));
        assert!(selector.applies(&Identifier::minecraft("block/oak_planks")));
        assert!(!selector.applies(&Identifier::minecraft("block/spruce_planks")));
    }
}
