pub mod atlas;
pub mod blockstate;
pub mod model;
pub mod texture;

pub trait LoadableAsset<T = Self> {
    fn load_asset<R: AsRef<str>>(raw: R) -> anyhow::Result<T>;
}
