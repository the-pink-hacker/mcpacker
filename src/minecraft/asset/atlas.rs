use std::path::PathBuf;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{
    types::identifier::{AssetType, Identifier},
    Asset,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Atlas {
    sources: Vec<AtlasSource>,
}

impl Asset for Atlas {
    fn get_type() -> AssetType {
        AssetType::Atlas
    }
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AtlasSource {
    Directory {
        source: PathBuf,
        prefix: String,
    },
    Single {
        resource: PathBuf,
        sprite: Option<String>,
    },
    Filter {
        namespace: Option<String>,
        path: Option<PathBuf>,
    },
    Unstich {
        resource: PathBuf,
        divisor_x: f64,
        divisor_y: f64,
        regions: Vec<()>,
    },
    PalettedPermutations {
        textures: Vec<Identifier>,
        palette_key: Vec<Identifier>,
        permutations: IndexMap<String, Identifier>,
    },
}
