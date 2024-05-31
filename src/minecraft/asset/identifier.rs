use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Identifier {
    namespace: String,
    path: PathBuf,
}
