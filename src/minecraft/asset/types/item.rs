use crate::{compile::modifier::zfighting::Direction, minecraft::serialize::*};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, OneOrMany};

use super::identifier::Identifier;

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemModelDefinition {
    #[serde(default = "get_true", skip_serializing_if = "is_true")]
    pub hand_animation_on_swap: bool,
    pub model: ModelType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ModelType {
    #[serde(rename = "minecraft:model")]
    Model {
        model: Identifier,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tints: Vec<TintSource>,
    },
    #[serde(rename = "minecraft:composite")]
    Composite { models: Vec<ModelType> },
    #[serde(rename = "minecraft:condition")]
    Condition {
        #[serde(flatten)]
        property: ConditionProperty,
        on_true: Box<ModelType>,
        on_false: Box<ModelType>,
    },
    #[serde(rename = "minecraft:select")]
    Select {
        #[serde(flatten)]
        property: SelectProperty,
        fallback: Box<ModelType>,
    },
    #[serde(rename = "minecraft:range_dispatch")]
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
    #[serde(rename = "minecraft:empty")]
    Empty,
    #[serde(rename = "minecraft:bundle/selected_item")]
    BundleSelectedItem,
    #[serde(rename = "minecraft:special")]
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
#[serde(tag = "type")]
pub enum TintSource {
    #[serde(rename = "minecraft:constant")]
    Constant { value: ColorRGB },
    #[serde(rename = "minecraft:dye")]
    Dye(DefaultColor),
    #[serde(rename = "minecraft:grass")]
    Grass(DefaultColor),
    #[serde(rename = "minecraft:firework")]
    Firework(DefaultColor),
    #[serde(rename = "minecraft:potion")]
    Potion(DefaultColor),
    #[serde(rename = "minecraft:map_color")]
    MapColor(DefaultColor),
    #[serde(rename = "minecraft:team")]
    Team(DefaultColor),
    #[serde(rename = "minecraft:custom_model_data")]
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
#[serde(tag = "property")]
pub enum ConditionProperty {
    #[serde(rename = "minecraft:using_item")]
    UsingItem,
    #[serde(rename = "minecraft:broken")]
    Broken,
    #[serde(rename = "minecraft:damaged")]
    Damaged,
    #[serde(rename = "minecraft:has_component")]
    HasComponent {
        component: String,
        #[serde(default, skip_serializing_if = "is_false")]
        ignore_default: bool,
    },
    #[serde(rename = "minecraft:fishing_rod/cast")]
    FishingRodCast,
    #[serde(rename = "minecraft:bundle/has_selected_item")]
    BundleHasSelectedItem,
    #[serde(rename = "minecraft:selected")]
    Selected,
    #[serde(rename = "minecraft:carried")]
    Carried,
    #[serde(rename = "minecraft:extended_view")]
    ExtendedView,
    #[serde(rename = "minecraft:keybind_down")]
    KeybindDown { keybind: String },
    #[serde(rename = "minecraft:view_entity")]
    ViewEntity,
    #[serde(rename = "minecraft:custom_model_data")]
    CustomModelData(CustomModelData),
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "property")]
pub enum SelectProperty {
    #[serde(rename = "minecraft:main_hand")]
    MainHand { cases: Vec<SelectCase<MainHand>> },
    #[serde(rename = "minecraft:charge_type")]
    ChargeType { cases: Vec<SelectCase<ChargeType>> },
    #[serde(rename = "minecraft:trim_material")]
    TrimMaterial { cases: Vec<SelectCase<Identifier>> },
    #[serde(rename = "minecraft:block_state")]
    BlockState {
        cases: Vec<SelectCase<String>>,
        block_state_property: String,
    },
    #[serde(rename = "minecraft:display_context")]
    DisplayContext {
        cases: Vec<SelectCase<DisplayContext>>,
    },
    #[serde(rename = "minecraft:local_time")]
    LocalTime {
        cases: Vec<SelectCase<String>>,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        locale: String,
        time_zone: Option<String>,
        pattern: String,
    },
    #[serde(rename = "minecraft:context_dimension")]
    ContextDimension { cases: Vec<SelectCase<Identifier>> },
    #[serde(rename = "minecraft:context_entity_type")]
    ContextEntityType { cases: Vec<SelectCase<Identifier>> },
    #[serde(rename = "minecraft:custom_model_data")]
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
#[serde(tag = "property")]
pub enum NumericProperty {
    #[serde(rename = "minecraft:bundle/fullness")]
    BundleFullness,
    #[serde(rename = "minecraft:damage")]
    Damage {
        #[serde(default = "get_true", skip_serializing_if = "is_true")]
        normalize: bool,
    },
    #[serde(rename = "minecraft:count")]
    Count {
        #[serde(default = "get_true", skip_serializing_if = "is_true")]
        normalize: bool,
    },
    #[serde(rename = "minecraft:cooldown")]
    Cooldown,
    #[serde(rename = "minecraft:time")]
    Time {
        source: TimeSource,
        #[serde(default = "get_true", skip_serializing_if = "is_true")]
        wobble: bool,
    },
    #[serde(rename = "minecraft:compass")]
    Compass {
        target: CompassTarget,
        #[serde(default = "get_true", skip_serializing_if = "is_true")]
        wobble: bool,
    },
    #[serde(rename = "minecraft:crossbow/pull")]
    CrossbowPull,
    #[serde(rename = "minecraft:use_duration")]
    UseDuration {
        #[serde(default = "get_true", skip_serializing_if = "is_true")]
        remaining: bool,
    },
    #[serde(rename = "minecraft:use_cycle")]
    UseCycle {
        #[serde(
            default = "NumericProperty::get_default_use_cycle_period",
            skip_serializing_if = "NumericProperty::is_use_cycle_period_default"
        )]
        period: f32,
    },
    #[serde(rename = "minecraft:custom_model_data")]
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
#[serde(tag = "type")]
pub enum SpecialModel {
    #[serde(rename = "minecraft:bed")]
    Bed { texture: Identifier },
    #[serde(rename = "minecraft:banner")]
    Banner { color: DyeColor },
    #[serde(rename = "minecraft:conduit")]
    Conduit,
    #[serde(rename = "minecraft:chest")]
    Chest {
        texture: Identifier,
        #[serde(default, skip_serializing_if = "SpecialModel::is_openness_default")]
        openness: f32,
    },
    #[serde(rename = "minecraft:decorated_pot")]
    DecoratedPot,
    #[serde(rename = "minecraft:head")]
    Head {
        kind: HeadKind,
        texture: Option<Identifier>,
        #[serde(
            default,
            skip_serializing_if = "SpecialModel::is_head_animation_default"
        )]
        animation: f32,
    },
    #[serde(rename = "minecraft:shulker_box")]
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
    #[serde(rename = "minecraft:shield")]
    Shield,
    #[serde(rename = "minecraft:standing_sign")]
    StandingSign(SignModel),
    #[serde(rename = "minecraft:hanging_sign")]
    HangingSign(SignModel),
    #[serde(rename = "minecraft:trident")]
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
