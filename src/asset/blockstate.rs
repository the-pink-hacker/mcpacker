use crate::minecraft::asset::blockstate::Blockstate;

use super::LoadableAsset;

impl LoadableAsset for Blockstate {
    fn load_asset<R: AsRef<str>>(raw: R) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(raw.as_ref())?)
    }
}
