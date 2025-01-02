use crate::{
    asset::LoadableAsset,
    compile::modifier::zfighting::Direction,
    minecraft::{asset::Asset, serialize::*},
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, OneOrMany};

use super::identifier::{AssetType, Identifier};

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemModelDefinition {
    #[serde(default = "get_true", skip_serializing_if = "is_true")]
    pub hand_animation_on_swap: bool,
    pub model: ModelType,
}

impl Asset for ItemModelDefinition {
    fn get_type() -> AssetType {
        AssetType::ItemModelDefinition
    }
}

impl LoadableAsset for ItemModelDefinition {
    fn load_asset<R: AsRef<str>>(raw: R) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(raw.as_ref())?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelType {
    #[serde(alias = "minecraft:model")]
    Model {
        model: Identifier,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tints: Vec<TintSource>,
    },
    #[serde(alias = "minecraft:composite")]
    Composite { models: Vec<ModelType> },
    #[serde(alias = "minecraft:condition")]
    Condition {
        #[serde(flatten)]
        property: ConditionProperty,
        on_true: Box<ModelType>,
        on_false: Box<ModelType>,
    },
    #[serde(alias = "minecraft:select")]
    Select {
        #[serde(flatten)]
        property: SelectProperty,
        fallback: Box<ModelType>,
    },
    #[serde(alias = "minecraft:range_dispatch")]
    RangeDispatch {
        #[serde(flatten)]
        property: NumericProperty,
        #[serde(
            default = "ModelType::get_default_range_dispatch_scale",
            skip_serializing_if = "ModelType::is_range_dispatch_scale_default"
        )]
        scale: f32,
        entries: Vec<RangeDispatchEntry>,
        fallback: Box<ModelType>,
    },
    #[serde(alias = "minecraft:empty")]
    Empty,
    #[serde(
        rename = "bundle/selected_item",
        alias = "minecraft:bundle/selected_item"
    )]
    BundleSelectedItem,
    #[serde(alias = "minecraft:special")]
    Special {
        model: SpecialModel,
        base: Identifier,
    },
}

impl ModelType {
    fn get_default_range_dispatch_scale() -> f32 {
        1.0
    }

    fn is_range_dispatch_scale_default(value: &f32) -> bool {
        *value == 1.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TintSource {
    #[serde(alias = "minecraft:constant")]
    Constant { value: ColorRGB },
    #[serde(alias = "minecraft:dye")]
    Dye(DefaultColor),
    #[serde(alias = "minecraft:grass")]
    Grass(DefaultColor),
    #[serde(alias = "minecraft:firework")]
    Firework(DefaultColor),
    #[serde(alias = "minecraft:potion")]
    Potion(DefaultColor),
    #[serde(alias = "minecraft:map_color")]
    MapColor(DefaultColor),
    #[serde(alias = "minecraft:team")]
    Team(DefaultColor),
    #[serde(alias = "minecraft:custom_model_data")]
    CustomModelData {
        #[serde(flatten)]
        custom_model_data: CustomModelData,
        default: ColorRGB,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultColor {
    default: ColorRGB,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColorRGB {
    Packed(i32),
    Array([f32; 3]),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "property", rename_all = "snake_case")]
pub enum ConditionProperty {
    #[serde(alias = "minecraft:using_item")]
    UsingItem,
    #[serde(alias = "minecraft:broken")]
    Broken,
    #[serde(alias = "minecraft:damaged")]
    Damaged,
    #[serde(alias = "minecraft:has_component")]
    HasComponent {
        component: String,
        #[serde(default, skip_serializing_if = "is_false")]
        ignore_default: bool,
    },
    #[serde(alias = "minecraft:fishing_rod/cast")]
    FishingRodCast,
    #[serde(
        rename = "bundle/has_selected_item",
        alias = "minecraft:bundle/has_selected_item"
    )]
    BundleHasSelectedItem,
    #[serde(alias = "minecraft:selected")]
    Selected,
    #[serde(alias = "minecraft:carried")]
    Carried,
    #[serde(alias = "minecraft:extended_view")]
    ExtendedView,
    #[serde(alias = "minecraft:keybind_down")]
    KeybindDown { keybind: String },
    #[serde(alias = "minecraft:view_entity")]
    ViewEntity,
    #[serde(alias = "minecraft:custom_model_data")]
    CustomModelData(CustomModelData),
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "property", rename_all = "snake_case")]
pub enum SelectProperty {
    #[serde(alias = "minecraft:main_hand")]
    MainHand { cases: Vec<SelectCase<MainHand>> },
    #[serde(alias = "minecraft:charge_type")]
    ChargeType { cases: Vec<SelectCase<ChargeType>> },
    #[serde(alias = "minecraft:trim_material")]
    TrimMaterial { cases: Vec<SelectCase<Identifier>> },
    #[serde(alias = "minecraft:block_state")]
    BlockState {
        cases: Vec<SelectCase<String>>,
        block_state_property: String,
    },
    #[serde(alias = "minecraft:display_context")]
    DisplayContext {
        cases: Vec<SelectCase<DisplayContext>>,
    },
    #[serde(alias = "minecraft:local_time")]
    LocalTime {
        cases: Vec<SelectCase<String>>,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        locale: String,
        time_zone: Option<String>,
        pattern: String,
    },
    #[serde(alias = "minecraft:context_dimension")]
    ContextDimension { cases: Vec<SelectCase<Identifier>> },
    #[serde(alias = "minecraft:context_entity_type")]
    ContextEntityType { cases: Vec<SelectCase<Identifier>> },
    #[serde(alias = "minecraft:custom_model_data")]
    CustomModelData {
        cases: Vec<SelectCase<String>>,
        #[serde(flatten)]
        custom_model_data: CustomModelData,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MainHand {
    Left,
    Right,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChargeType {
    None,
    Rocket,
    Arrow,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplayContext {
    None,
    ThirdpersonLefthand,
    ThirdpersonRighthand,
    FirstpersonLefthand,
    FirstpersonRighthand,
    Head,
    Gui,
    Ground,
    Fixed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomModelData {
    #[serde(default, skip_serializing_if = "CustomModelData::is_default")]
    index: i32,
}

impl CustomModelData {
    fn is_default(value: &i32) -> bool {
        *value == 0
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct SelectCase<T: Serialize + DeserializeOwned> {
    #[serde_as(as = "OneOrMany<_>")]
    when: Vec<T>,
    model: ModelType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RangeDispatchEntry {
    threshold: f32,
    model: ModelType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "property", rename_all = "snake_case")]
pub enum NumericProperty {
    #[serde(rename = "bundle/fullness", alias = "minecraft:bundle/fullness")]
    BundleFullness,
    #[serde(alias = "minecraft:damage")]
    Damage {
        #[serde(default = "get_true", skip_serializing_if = "is_true")]
        normalize: bool,
    },
    #[serde(alias = "minecraft:count")]
    Count {
        #[serde(default = "get_true", skip_serializing_if = "is_true")]
        normalize: bool,
    },
    #[serde(alias = "minecraft:cooldown")]
    Cooldown,
    #[serde(alias = "minecraft:time")]
    Time {
        source: TimeSource,
        #[serde(default = "get_true", skip_serializing_if = "is_true")]
        wobble: bool,
    },
    #[serde(alias = "minecraft:compass")]
    Compass {
        target: CompassTarget,
        #[serde(default = "get_true", skip_serializing_if = "is_true")]
        wobble: bool,
    },
    #[serde(rename = "crossbow/pull", alias = "minecraft:crossbow/pull")]
    CrossbowPull,
    #[serde(alias = "minecraft:use_duration")]
    UseDuration {
        #[serde(default = "get_true", skip_serializing_if = "is_true")]
        remaining: bool,
    },
    #[serde(alias = "minecraft:use_cycle")]
    UseCycle {
        #[serde(
            default = "NumericProperty::get_default_use_cycle_period",
            skip_serializing_if = "NumericProperty::is_use_cycle_period_default"
        )]
        period: f32,
    },
    #[serde(alias = "minecraft:custom_model_data")]
    CustomModelData(CustomModelData),
}

impl NumericProperty {
    fn get_default_use_cycle_period() -> f32 {
        1.0
    }

    fn is_use_cycle_period_default(value: &f32) -> bool {
        *value == 1.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeSource {
    Daytime,
    MoonPhase,
    Random,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompassTarget {
    Spawn,
    Lodestone,
    Recovery,
    None,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SpecialModel {
    #[serde(alias = "minecraft:bed")]
    Bed { texture: Identifier },
    #[serde(alias = "minecraft:banner")]
    Banner { color: DyeColor },
    #[serde(alias = "minecraft:conduit")]
    Conduit,
    #[serde(alias = "minecraft:chest")]
    Chest {
        texture: Identifier,
        #[serde(default, skip_serializing_if = "SpecialModel::is_openness_default")]
        openness: f32,
    },
    #[serde(alias = "minecraft:decorated_pot")]
    DecoratedPot,
    #[serde(alias = "minecraft:head")]
    Head {
        kind: HeadKind,
        texture: Option<Identifier>,
        #[serde(
            default,
            skip_serializing_if = "SpecialModel::is_head_animation_default"
        )]
        animation: f32,
    },
    #[serde(alias = "minecraft:shulker_box")]
    ShulkerBox {
        texture: Identifier,
        #[serde(default, skip_serializing_if = "SpecialModel::is_openness_default")]
        openness: f32,
        #[serde(
            default = "SpecialModel::get_default_shulker_box_orientation",
            skip_serializing_if = "SpecialModel::is_shulker_box_orientation_default"
        )]
        orientation: Direction,
    },
    #[serde(alias = "minecraft:shield")]
    Shield,
    #[serde(alias = "minecraft:standing_sign")]
    StandingSign(SignModel),
    #[serde(alias = "minecraft:hanging_sign")]
    HangingSign(SignModel),
    #[serde(alias = "minecraft:trident")]
    Trident,
}

impl SpecialModel {
    fn is_openness_default(value: &f32) -> bool {
        *value == 0.0
    }

    fn is_head_animation_default(value: &f32) -> bool {
        *value == 0.0
    }

    fn get_default_shulker_box_orientation() -> Direction {
        Direction::Up
    }

    fn is_shulker_box_orientation_default(value: &Direction) -> bool {
        *value == Direction::Up
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DyeColor {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeadKind {
    Skeleton,
    WitherSkeleton,
    Player,
    Zombie,
    Creeper,
    Piglin,
    Dragon,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignModel {
    wood_type: SignWoodType,
    texture: Identifier,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignWoodType {
    Oak,
    Spruce,
    Birch,
    Acacia,
    Cherry,
    Jungle,
    DarkOak,
    PaleOak,
    Mangrove,
    Bamboo,
    Crimson,
    Warped,
}
