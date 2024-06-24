use self::types::identifier::AssetType;

pub mod atlas;
pub mod blockstate;
pub mod model;
pub mod texture;
pub mod types;

pub trait Asset {
    fn get_type() -> AssetType;

    fn is_virtual(&self) -> bool {
        false
    }
}
