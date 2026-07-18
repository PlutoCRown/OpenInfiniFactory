mod adapter;
#[macro_use]
mod register;
mod material_catalog;
mod material_props;
mod paint_catalog;
mod registry;
mod scene_catalog;
mod stamp_catalog;
pub mod traits;

pub mod blocker;
pub mod converter;
pub mod conveyor;
pub mod counter_rotator;
pub mod detector;
pub mod down_detector;
pub mod down_welder;
pub mod drill;
pub mod drill_head;
pub mod generator;
pub mod goal;
pub mod laser;
pub mod lifter;
pub mod mirror;
pub mod platform;
pub mod pusher;
pub mod reverse_conveyor;
pub mod roller;
pub mod roller_body;
pub mod rotator;
pub mod splitter;
pub mod stamper;
pub mod stamper_body;
pub mod sign;
pub mod suction_cup;
pub mod teleport_entrance;
pub mod teleport_exit;
pub mod vertical_mirror;
pub mod weld_point;
pub mod welder;
pub mod wire;

use glam::IVec3;
use serde::{Deserialize, Serialize};

pub use self::material_catalog::{
    ensure_fallback_material_catalog, install_material_catalog, material_catalog, material_def,
    MaterialBlockCatalog, MaterialBlockDef, MaterialBlockId,
};
pub use self::material_props::{
    local_face_index, material_face_connectable, MaterialProps,
};
pub use self::paint_catalog::{
    ensure_fallback_paint_catalog, install_paint_catalog, paint_catalog, paint_def,
    PaintMaterialCatalog, PaintMaterialDef, PaintMaterialId,
};
pub use self::registry::{assert_registry_consistent, save_stores_facing};
pub use self::scene_catalog::{
    ensure_fallback_scene_catalog, install_scene_catalog, leak_str, scene_catalog, scene_def,
    SceneBlockCatalog, SceneBlockDef, SceneBlockId,
};
pub use self::stamp_catalog::{
    ensure_fallback_stamp_catalog, install_stamp_catalog, stamp_catalog, stamp_def,
    StampMaterialCatalog, StampMaterialDef, StampMaterialId,
};
pub use crate::world::direction::Facing;
use crate::world::grid::BlockSettings;

pub const BLOCK_SIZE: f32 = 1.0;
pub const DEFAULT_GENERATOR_PERIOD: u64 = 3;

/// 模拟侧注册表对象：Meta + Behavior
pub trait Block: traits::BlockMeta + traits::BlockBehavior {}

impl<T> Block for T where T: traits::BlockMeta + traits::BlockBehavior {}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PersistentLayer {
    Puzzle,
    SolutionFactory,
}

#[derive(Clone, Copy)]
pub struct BlockDefinition {
    pub kind: BlockKind,
    pub name_key: &'static str,
    pub short_name_key: &'static str,
    pub description_key: &'static str,
    color: ColorSpec,
    class: BlockClass,
    persistence: Option<PersistentLayer>,
    shape: BlockShape,
    collision: bool,
    transparent: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockClass {
    Scene,
    Factory,
    Material,
    System,
    Virtual,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum BlockLayer {
    Scene(SceneBlockId),
    Material(MaterialBlock),
    Factory(FactoryBlock),
    System(SystemBlock),
    Virtual(VirtualBlock),
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum MaterialBlock {
    Material(MaterialBlockId),
    Stamp(StampMaterialId),
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum FactoryBlock {
    Platform,
    Welder,
    DownWelder,
    Conveyor,
    ReverseConveyor,
    Detector,
    DownDetector,
    Wire,
    Pusher,
    Lifter,
    Rotator,
    CounterRotator,
    Blocker,
    Drill,
    Laser,
    Mirror,
    VerticalMirror,
    Splitter,
    SuctionCup,
    Sign,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SystemBlock {
    Generator,
    Goal,
    Stamper,
    Roller,
    Converter,
    TeleportEntrance,
    TeleportExit,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum VirtualBlock {
    WeldPoint,
    DrillHead,
    /// 滚刷机实体占格（有碰撞，写入 machine_bodies）
    RollerBody,
    /// 印花机实体占格（有碰撞，写入 machine_bodies；朝向与宿主同步）
    StamperBody,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum BlockShape {
    Cube,
    Node,
}

#[derive(Clone, Copy)]
pub enum MarkerBehavior {
    WeldPoint { offset: IVec3, facing: Facing },
    DrillHead { offset: IVec3, facing: Facing },
    /// 滚刷机同格实体占位
    RollerBody { facing: Facing },
    /// 印花机同格实体占位（朝向与宿主一致，供 L4 透传）
    StamperBody { facing: Facing },
}

#[derive(Clone, Copy)]
pub enum MaterialSource {
    Generator,
}

/// 材料移动规则：传送带 / 升降 / 旋转 / 通电伸缩
#[derive(Clone, Copy)]
pub enum MovementRule {
    Translate {
        source: IVec3,
        offset: IVec3,
    },
    Lift {
        range: i32,
    },
    Rotate {
        clockwise: bool,
    },
    /// 通电伸缩：`extend_when_powered` 为真则通电伸出，为假则通电收回
    PoweredTranslate {
        source: IVec3,
        offset: IVec3,
        extend_when_powered: bool,
    },
}

/// 材料销毁方式：钻头 / 邻接钻头 / 激光
#[derive(Clone, Copy)]
pub enum MaterialDestroyer {
    Drill { target: IVec3 },
    AdjacentDrillHead,
    Laser { direction: IVec3, range: i32 },
}

/// 材料打标方式：印花 / 滚筒
#[derive(Clone, Copy)]
pub enum MaterialLabeler {
    Stamper { target: IVec3 },
    Roller { target: IVec3 },
}

/// 材料处理器：转换器 / 传送入口
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaterialProcessor {
    Converter,
    TeleportEntrance,
}

/// 激光光学行为：平面镜 / 垂直镜 / 分光镜
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaserOpticsBehavior {
    Mirror,
    VerticalMirror,
    Splitter,
}

/// 焊接行为：焊点节点
#[derive(Clone, Copy)]
pub enum WeldBehavior {
    Node,
}

/// 信号行为：导线 / 传感器 / 用电器
#[derive(Clone, Copy)]
pub enum SignalBehavior {
    Wire,
    Detector { detection_pos: IVec3 },
    PoweredDevice,
}

/// 方块目录用 RGBA 颜色规格（表现层再转 Bevy Color）
#[derive(Clone, Copy, Debug)]
pub struct ColorSpec {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub const fn rgb(r: f32, g: f32, b: f32) -> ColorSpec {
    ColorSpec { r, g, b, a: 1.0 }
}

pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> ColorSpec {
    ColorSpec { r, g, b, a }
}

impl BlockDefinition {
    pub const fn scene(
        kind: BlockKind,
        name_key: &'static str,
        short_name_key: &'static str,
        description_key: &'static str,
        color: ColorSpec,
    ) -> Self {
        Self::new(
            kind,
            name_key,
            short_name_key,
            description_key,
            color,
            BlockClass::Scene,
            Some(PersistentLayer::Puzzle),
        )
    }

    pub const fn factory(
        kind: BlockKind,
        name_key: &'static str,
        short_name_key: &'static str,
        description_key: &'static str,
        color: ColorSpec,
    ) -> Self {
        Self::new(
            kind,
            name_key,
            short_name_key,
            description_key,
            color,
            BlockClass::Factory,
            Some(PersistentLayer::SolutionFactory),
        )
    }

    pub const fn material(
        kind: BlockKind,
        name_key: &'static str,
        short_name_key: &'static str,
        description_key: &'static str,
        color: ColorSpec,
    ) -> Self {
        Self::new(
            kind,
            name_key,
            short_name_key,
            description_key,
            color,
            BlockClass::Material,
            None,
        )
    }

    pub const fn marker(
        kind: BlockKind,
        name_key: &'static str,
        short_name_key: &'static str,
        description_key: &'static str,
        color: ColorSpec,
    ) -> Self {
        Self::new(
            kind,
            name_key,
            short_name_key,
            description_key,
            color,
            BlockClass::Virtual,
            None,
        )
    }

    pub const fn puzzle_system(
        kind: BlockKind,
        name_key: &'static str,
        short_name_key: &'static str,
        description_key: &'static str,
        color: ColorSpec,
    ) -> Self {
        Self::new(
            kind,
            name_key,
            short_name_key,
            description_key,
            color,
            BlockClass::System,
            Some(PersistentLayer::Puzzle),
        )
    }

    const fn new(
        kind: BlockKind,
        name_key: &'static str,
        short_name_key: &'static str,
        description_key: &'static str,
        color: ColorSpec,
        class: BlockClass,
        persistence: Option<PersistentLayer>,
    ) -> Self {
        Self {
            kind,
            name_key,
            short_name_key,
            description_key,
            color,
            class,
            persistence,
            shape: BlockShape::Cube,
            collision: true,
            transparent: false,
        }
    }

    pub const fn node(mut self) -> Self {
        self.shape = BlockShape::Node;
        self
    }

    pub const fn no_collision(mut self) -> Self {
        self.collision = false;
        self
    }

    pub const fn with_collision(mut self, collision: bool) -> Self {
        self.collision = collision;
        self
    }

    pub const fn transparent(mut self) -> Self {
        self.transparent = true;
        self
    }

    pub fn color(self) -> ColorSpec {
        self.color
    }

    pub fn class(self) -> BlockClass {
        self.class
    }

    pub fn shape(self) -> BlockShape {
        self.shape
    }

    pub fn is_transparent(self) -> bool {
        self.transparent
    }
}

impl BlockLayer {
    pub fn class(self) -> BlockClass {
        match self {
            Self::Scene(_) => BlockClass::Scene,
            Self::Material(_) => BlockClass::Material,
            Self::Factory(_) => BlockClass::Factory,
            Self::System(_) => BlockClass::System,
            Self::Virtual(_) => BlockClass::Virtual,
        }
    }
}

/// 方块实例 ID：放置/加载时分配，移动时保持不变，供动画与场景实体追踪
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct BlockId(pub u64);

impl BlockId {
    pub const NONE: Self = Self(0);

    pub const fn is_none(self) -> bool {
        self.0 == 0
    }
}

/// 验收结构 ID：Goal 放置相连时分配，供生成器连接模式绑定
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
pub struct AcceptorId(pub u64);

impl AcceptorId {
    pub const NONE: Self = Self(0);

    pub const fn is_none(self) -> bool {
        self.0 == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockData {
    pub kind: BlockKind,
    pub facing: Facing,
    #[serde(default)]
    pub id: BlockId,
}

impl BlockData {
    pub const fn new(kind: BlockKind, facing: Facing) -> Self {
        Self {
            kind,
            facing,
            id: BlockId::NONE,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum BlockKind {
    /// 配置式场景方块（内置 grass/stone/dirt/planks 的 id 固定为 0..=3）
    Scene(SceneBlockId),
    Platform,
    Generator,
    Welder,
    DownWelder,
    Conveyor,
    ReverseConveyor,
    Detector,
    DownDetector,
    Wire,
    Pusher,
    Lifter,
    Rotator,
    CounterRotator,
    Blocker,
    Drill,
    Laser,
    Mirror,
    VerticalMirror,
    Splitter,
    SuctionCup,
    Sign,
    Stamper,
    Roller,
    Converter,
    TeleportEntrance,
    TeleportExit,
    Goal,
    /// 配置式材料方块（basic/iron/copper/glass 等）
    Material(MaterialBlockId),
    /// 配置式印花材料（red/green/blue/yellow 等）
    Stamp(StampMaterialId),
    WeldPoint,
    DrillHead,
    RollerBody,
    StamperBody,
}

impl BlockKind {
    /// 按资源包字符串 id 解析场景方块（依赖已安装的 catalog）
    pub fn scene(string_id: &str) -> Self {
        ensure_fallback_scene_catalog();
        let id = scene_catalog()
            .id_by_string(string_id)
            .unwrap_or_else(|| panic!("unknown scene block id `{string_id}`"));
        Self::Scene(id)
    }

    /// 按资源包字符串 id 解析材料方块
    pub fn material(string_id: &str) -> Self {
        ensure_fallback_material_catalog();
        let id = material_catalog()
            .id_by_string(string_id)
            .unwrap_or_else(|| panic!("unknown material block id `{string_id}`"));
        Self::Material(id)
    }

    /// 按资源包字符串 id 解析印花材料
    pub fn stamp(string_id: &str) -> Self {
        ensure_fallback_stamp_catalog();
        let id = stamp_catalog()
            .id_by_string(string_id)
            .unwrap_or_else(|| panic!("unknown stamp material id `{string_id}`"));
        Self::Stamp(id)
    }

    /// 由材料 catalog id 构造 BlockKind
    pub fn material_block_kind(id: MaterialBlockId) -> Self {
        Self::Material(id)
    }

    /// 由印花 catalog id 构造 BlockKind
    pub fn stamp_block_kind(id: StampMaterialId) -> Self {
        Self::Stamp(id)
    }

    fn block(self) -> &'static (dyn Block + Send + Sync) {
        match self {
            BlockKind::Scene(_) | BlockKind::Material(_) | BlockKind::Stamp(_) => {
                panic!("Scene/Material/Stamp blocks are not inventory-registered")
            }
            other => registry::get(other),
        }
    }

    pub fn layer(self) -> BlockLayer {
        match self {
            BlockKind::Scene(id) => BlockLayer::Scene(id),
            BlockKind::Material(id) => BlockLayer::Material(MaterialBlock::Material(id)),
            BlockKind::Stamp(id) => BlockLayer::Material(MaterialBlock::Stamp(id)),
            BlockKind::Platform => BlockLayer::Factory(FactoryBlock::Platform),
            BlockKind::Welder => BlockLayer::Factory(FactoryBlock::Welder),
            BlockKind::DownWelder => BlockLayer::Factory(FactoryBlock::DownWelder),
            BlockKind::Conveyor => BlockLayer::Factory(FactoryBlock::Conveyor),
            BlockKind::ReverseConveyor => BlockLayer::Factory(FactoryBlock::ReverseConveyor),
            BlockKind::Detector => BlockLayer::Factory(FactoryBlock::Detector),
            BlockKind::DownDetector => BlockLayer::Factory(FactoryBlock::DownDetector),
            BlockKind::Wire => BlockLayer::Factory(FactoryBlock::Wire),
            BlockKind::Pusher => BlockLayer::Factory(FactoryBlock::Pusher),
            BlockKind::Lifter => BlockLayer::Factory(FactoryBlock::Lifter),
            BlockKind::Rotator => BlockLayer::Factory(FactoryBlock::Rotator),
            BlockKind::CounterRotator => BlockLayer::Factory(FactoryBlock::CounterRotator),
            BlockKind::Blocker => BlockLayer::Factory(FactoryBlock::Blocker),
            BlockKind::Drill => BlockLayer::Factory(FactoryBlock::Drill),
            BlockKind::Laser => BlockLayer::Factory(FactoryBlock::Laser),
            BlockKind::Mirror => BlockLayer::Factory(FactoryBlock::Mirror),
            BlockKind::VerticalMirror => BlockLayer::Factory(FactoryBlock::VerticalMirror),
            BlockKind::Splitter => BlockLayer::Factory(FactoryBlock::Splitter),
            BlockKind::SuctionCup => BlockLayer::Factory(FactoryBlock::SuctionCup),
            BlockKind::Sign => BlockLayer::Factory(FactoryBlock::Sign),
            BlockKind::Generator => BlockLayer::System(SystemBlock::Generator),
            BlockKind::Goal => BlockLayer::System(SystemBlock::Goal),
            BlockKind::Stamper => BlockLayer::System(SystemBlock::Stamper),
            BlockKind::Roller => BlockLayer::System(SystemBlock::Roller),
            BlockKind::Converter => BlockLayer::System(SystemBlock::Converter),
            BlockKind::TeleportEntrance => BlockLayer::System(SystemBlock::TeleportEntrance),
            BlockKind::TeleportExit => BlockLayer::System(SystemBlock::TeleportExit),
            BlockKind::WeldPoint => BlockLayer::Virtual(VirtualBlock::WeldPoint),
            BlockKind::DrillHead => BlockLayer::Virtual(VirtualBlock::DrillHead),
            BlockKind::RollerBody => BlockLayer::Virtual(VirtualBlock::RollerBody),
            BlockKind::StamperBody => BlockLayer::Virtual(VirtualBlock::StamperBody),
        }
    }

    pub fn definition(self) -> BlockDefinition {
        match self {
            BlockKind::Scene(id) => {
                let def = scene_def(id);
                BlockDefinition::scene(
                    self,
                    def.name_key,
                    def.short_name_key,
                    def.description_key,
                    def.color,
                )
                .with_collision(def.collision)
            }
            BlockKind::Material(id) => {
                let def = material_def(id);
                BlockDefinition::material(
                    self,
                    def.name_key,
                    def.short_name_key,
                    def.description_key,
                    def.color,
                )
            }
            BlockKind::Stamp(id) => {
                let def = stamp_def(id);
                BlockDefinition::material(
                    self,
                    def.name_key,
                    def.short_name_key,
                    def.description_key,
                    def.color,
                )
            }
            other => other.block().definition(),
        }
    }

    pub fn name_key(self) -> &'static str {
        self.definition().name_key
    }

    pub fn short_name_key(self) -> &'static str {
        self.definition().short_name_key
    }

    pub fn description_key(self) -> &'static str {
        self.definition().description_key
    }

    pub fn shape(self) -> BlockShape {
        self.definition().shape()
    }

    pub fn is_transparent(self) -> bool {
        self.definition().is_transparent()
    }

    pub fn is_directional(self) -> bool {
        match self {
            BlockKind::Scene(id) => scene_def(id).directional,
            BlockKind::Material(id) => material_def(id).directional,
            BlockKind::Stamp(_) => false,
            other => other.block().is_directional(),
        }
    }

    /// 材料静态属性；非材料返回 None
    pub fn material_props(self) -> Option<MaterialProps> {
        match self {
            BlockKind::Material(id) => Some(MaterialProps::from_material_def(&material_def(id))),
            BlockKind::Stamp(id) => Some(MaterialProps::stamp_props(stamp_def(id).fragile)),
            _ => None,
        }
    }

    /// 仅 `BlockKind::Material` 返回 catalog id
    pub fn material_id(self) -> Option<MaterialBlockId> {
        match self {
            BlockKind::Material(id) => Some(id),
            _ => None,
        }
    }

    /// 仅 `BlockKind::Stamp` 返回 catalog id
    pub fn stamp_id(self) -> Option<StampMaterialId> {
        match self {
            BlockKind::Stamp(id) => Some(id),
            _ => None,
        }
    }

    /// 材料世界法线面是否 Connectable
    pub fn material_face_connectable(self, facing: Facing, world_normal: IVec3) -> bool {
        self.material_props()
            .is_some_and(|props| material_face_connectable(props, facing, world_normal))
    }

    /// 工厂可贴面：非 `non_connection_face`；场景读 catalog.connectable
    pub fn face_attachable(self, facing: Facing, world_normal: IVec3) -> bool {
        if let BlockKind::Scene(id) = self {
            let def = scene_def(id);
            let local = facing.inverse_rotate_offset(world_normal);
            return local_face_index(local).is_some_and(|i| def.connectable[i]);
        }
        if self.is_factory() {
            return self.non_connection_face(facing) != Some(world_normal);
        }
        false
    }

    pub fn has_collision(self) -> bool {
        if let BlockKind::Scene(id) = self {
            return scene_def(id).collision;
        }
        self.definition().collision
    }

    pub fn blocks_laser(self) -> bool {
        self.has_collision() && !self.is_material()
    }

    pub fn is_factory(self) -> bool {
        matches!(self.layer(), BlockLayer::Factory(_))
    }

    pub fn is_detectable_by_detector(self) -> bool {
        self.is_material() || matches!(self, BlockKind::Platform)
    }

    pub fn is_scene(self) -> bool {
        matches!(self.layer(), BlockLayer::Scene(_))
    }

    pub fn is_material(self) -> bool {
        matches!(self.layer(), BlockLayer::Material(_))
    }

    pub fn is_generated_marker(self) -> bool {
        matches!(self.layer(), BlockLayer::Virtual(_))
    }

    pub fn is_system_block(self) -> bool {
        matches!(self.layer(), BlockLayer::System(_))
    }

    /// 写入 system_blocks：系统方块，或无碰撞虚拟 marker（可与材料同格）
    pub fn is_system_layer(self) -> bool {
        self.is_system_block() || (self.is_generated_marker() && !self.has_collision())
    }

    pub fn accepts_material(self) -> bool {
        matches!(self, BlockKind::Goal)
    }

    pub fn shows_material_preview(self) -> bool {
        matches!(self, BlockKind::Generator | BlockKind::Goal)
    }

    pub fn material_shell_scale(self) -> f32 {
        if self.is_system_block() && self.persistent_layer() == Some(PersistentLayer::Puzzle) {
            1.05
        } else {
            1.0
        }
    }

    pub fn alternate(self) -> Option<Self> {
        if matches!(
            self,
            BlockKind::Scene(_) | BlockKind::Material(_) | BlockKind::Stamp(_)
        ) {
            return None;
        }
        self.block().alternate()
    }

    pub fn marker_behavior(self, facing: Facing) -> Option<MarkerBehavior> {
        if matches!(
            self,
            BlockKind::Scene(_) | BlockKind::Material(_) | BlockKind::Stamp(_)
        ) {
            return None;
        }
        self.block().marker_behavior(facing)
    }

    pub fn material_source(self, facing: Facing) -> Option<MaterialSource> {
        if matches!(
            self,
            BlockKind::Scene(_) | BlockKind::Material(_) | BlockKind::Stamp(_)
        ) {
            return None;
        }
        self.block().material_source(facing)
    }

    pub fn persistent_layer(self) -> Option<PersistentLayer> {
        if matches!(self, BlockKind::Scene(_)) {
            return Some(PersistentLayer::Puzzle);
        }
        if matches!(self, BlockKind::Material(_) | BlockKind::Stamp(_)) {
            return None;
        }
        self.block().persistent_layer()
    }

    pub fn default_settings(self, pos: IVec3) -> Option<BlockSettings> {
        if matches!(
            self,
            BlockKind::Scene(_) | BlockKind::Material(_) | BlockKind::Stamp(_)
        ) {
            return None;
        }
        self.block().default_settings(pos)
    }

    pub fn movement_rule(self, facing: Facing) -> Option<MovementRule> {
        if matches!(
            self,
            BlockKind::Scene(_) | BlockKind::Material(_) | BlockKind::Stamp(_)
        ) {
            return None;
        }
        self.block().movement_rule(facing)
    }

    pub fn material_destroyer(self, facing: Facing) -> Option<MaterialDestroyer> {
        if matches!(
            self,
            BlockKind::Scene(_) | BlockKind::Material(_) | BlockKind::Stamp(_)
        ) {
            return None;
        }
        self.block().material_destroyer(facing)
    }

    pub fn material_labeler(self, facing: Facing) -> Option<MaterialLabeler> {
        if matches!(
            self,
            BlockKind::Scene(_) | BlockKind::Material(_) | BlockKind::Stamp(_)
        ) {
            return None;
        }
        self.block().material_labeler(facing)
    }

    pub fn material_processor(self) -> Option<MaterialProcessor> {
        if matches!(
            self,
            BlockKind::Scene(_) | BlockKind::Material(_) | BlockKind::Stamp(_)
        ) {
            return None;
        }
        self.block().material_processor()
    }

    pub fn laser_optics(self) -> Option<LaserOpticsBehavior> {
        if matches!(
            self,
            BlockKind::Scene(_) | BlockKind::Material(_) | BlockKind::Stamp(_)
        ) {
            return None;
        }
        self.block().laser_optics()
    }

    pub fn weld_behavior(self) -> Option<WeldBehavior> {
        if matches!(
            self,
            BlockKind::Scene(_) | BlockKind::Material(_) | BlockKind::Stamp(_)
        ) {
            return None;
        }
        self.block().weld_behavior()
    }

    pub fn signal_behavior(self, facing: Facing) -> Option<SignalBehavior> {
        if matches!(
            self,
            BlockKind::Scene(_) | BlockKind::Material(_) | BlockKind::Stamp(_)
        ) {
            return None;
        }
        self.block().signal_behavior(facing)
    }

    pub fn non_connection_face(self, facing: Facing) -> Option<IVec3> {
        if matches!(
            self,
            BlockKind::Scene(_) | BlockKind::Material(_) | BlockKind::Stamp(_)
        ) {
            return None;
        }
        self.block().non_connection_face(facing)
    }
}
