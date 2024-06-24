use crate::minecraft::asset::texture::TextureMeta;

use super::LoadableAsset;

impl LoadableAsset for TextureMeta {
    fn load_asset<R: AsRef<str>>(raw: R) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(raw.as_ref())?)
    }
}
