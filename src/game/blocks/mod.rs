mod adapter;
pub mod panels;
#[macro_use]
mod register;
mod basic;
mod registry;
pub mod traits;

mod blocker;
mod blocker_head;
mod converter;
mod conveyor;
mod copper_material;
mod counter_rotator;
mod detector;
mod dirt;
mod down_detector;
mod down_welder;
mod drill;
mod drill_head;
mod generator;
mod goal;
mod grass;
mod iron_material;
mod laser;
mod lifter;
mod material;
mod planks;
mod platform;
pub mod pusher;
mod reverse_conveyor;
mod roller;
mod rotator;
mod stamper;
mod stone;
mod teleport_entrance;
mod teleport_exit;
mod weld_point;
mod welder;
mod wire;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub use self::registry::{all_blocks, assert_registry_consistent, edit_blocks, PLAY_BLOCKS};
use crate::game::state::UiPanelId;
pub use crate::game::world::direction::Facing;
use crate::game::world::grid::BlockSettings;

pub const BLOCK_SIZE: f32 = 1.0;
pub const DEFAULT_GENERATOR_PERIOD: u64 = 3;

pub trait Block: Send + Sync {
    fn id(&self) -> BlockKind;
    fn definition(&self) -> BlockDefinition;

    fn is_directional(&self) -> bool {
        false
    }

    fn marker_behavior(&self, _facing: Facing) -> Option<MarkerBehavior> {
        None
    }

    fn material_source(&self, _facing: Facing) -> Option<MaterialSource> {
        None
    }

    fn material_kind(&self) -> Option<MaterialKind> {
        None
    }

    fn persistent_layer(&self) -> Option<PersistentLayer> {
        self.definition().persistence
    }

    fn default_settings(&self, _pos: IVec3) -> Option<BlockSettings> {
        None
    }

    fn movement_rule(&self, _facing: Facing) -> Option<MovementRule> {
        None
    }

    fn material_destroyer(&self, _facing: Facing) -> Option<MaterialDestroyer> {
        None
    }

    fn material_labeler(&self, _facing: Facing) -> Option<MaterialLabeler> {
        None
    }

    fn weld_behavior(&self) -> Option<WeldBehavior> {
        None
    }

    fn signal_behavior(&self, _facing: Facing) -> Option<SignalBehavior> {
        None
    }

    fn render_behavior(&self, _facing: Facing) -> RenderBehavior {
        RenderBehavior::default()
    }

    fn model(&self) -> BlockModel {
        BlockModel::Default
    }

    fn alternate(&self) -> Option<BlockKind> {
        None
    }
}

pub trait EditableBlock: Block {
    fn ui_panel(&self) -> Option<UiPanelId>;
}

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
    color: ColorSpec,
    slot_color: ColorSpec,
    class: BlockClass,
    persistence: Option<PersistentLayer>,
    shape: BlockShape,
    collision: bool,
    transparent: bool,
    texture: Option<BlockTexture>,
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
    Scene(SceneBlock),
    Material(MaterialBlock),
    Factory(FactoryBlock),
    System(SystemBlock),
    Virtual(VirtualBlock),
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum SceneBlock {
    Grass,
    Stone,
    Dirt,
    Planks,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum MaterialBlock {
    Material,
    IronMaterial,
    CopperMaterial,
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
    BlockerHead,
    DrillHead,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum BlockShape {
    Cube,
    Node,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum BlockTexture {
    Material,
    IronMaterial,
    CopperMaterial,
    Platform,
    Grass,
    Stone,
    Dirt,
    Wood,
    BorderedWood,
}

#[derive(Clone, Copy)]
pub enum MarkerBehavior {
    WeldPoint { offset: IVec3, facing: Facing },
    BlockerHead { offset: IVec3, facing: Facing },
    DrillHead { offset: IVec3, facing: Facing },
}

#[derive(Clone, Copy)]
pub enum MaterialSource {
    Generator,
}

#[derive(Clone, Copy)]
pub enum MovementRule {
    Translate { source: IVec3, offset: IVec3 },
    Lift { range: i32 },
    Rotate { clockwise: bool },
    PoweredTranslate { source: IVec3, offset: IVec3 },
}

#[derive(Clone, Copy)]
pub enum MaterialDestroyer {
    Drill { target: IVec3 },
    AdjacentDrillHead,
    Laser { direction: IVec3, range: i32 },
}

#[derive(Clone, Copy)]
pub enum MaterialLabeler {
    Stamper { target: IVec3 },
    Roller { target: IVec3 },
}

#[derive(Clone, Copy)]
pub enum WeldBehavior {
    Node,
}

#[derive(Clone, Copy)]
pub enum SignalBehavior {
    Wire,
    Detector { detection_pos: IVec3 },
    PoweredDevice,
}

#[derive(Clone, Copy, Default)]
pub struct RenderBehavior {
    pub weld_connector: Option<WeldConnectorBehavior>,
    pub wire_connector: Option<WireConnectorBehavior>,
}

#[derive(Clone, Copy)]
pub enum WeldConnectorBehavior {
    AllSides,
    Offset(IVec3),
}

#[derive(Clone, Copy)]
pub enum WireConnectorBehavior {
    Wire,
    Device { blocked_offset: IVec3 },
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum ModelMesh {
    ConveyorBase,
    ConveyorBelt,
    DrillBody,
    DrillTip,
    PusherBody,
    PusherHead,
    Large,
    Medium,
    Small,
    Plate,
    RotatorBase,
    RotatorDisk,
    RotatorRing,
    RodX,
    RodY,
    RodZ,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum ModelMaterial {
    ConveyorBase,
    ConveyorBelt,
    DrillTip,
    Frame,
    DarkFrame,
    Belt,
    BeltStripe,
    WeldCore,
    Welding,
    Wire,
    Signal,
    Power,
    Pusher,
    Platform,
    PlatformBase,
    Wood,
    WoodTexture,
    BorderedWoodTexture,
    StoneTexture,
    Lift,
    Rotation,
    Drill,
    Laser,
    System,
    SystemAccent,
    TeleportIn,
    TeleportOut,
}

#[derive(Clone, Copy)]
pub struct BlockModelPart {
    pub mesh: ModelMesh,
    pub material: ModelMaterial,
    pub translation: [f32; 3],
    pub scale: [f32; 3],
    pub yaw_radians: f32,
}

impl BlockModelPart {
    pub const fn new(mesh: ModelMesh, material: ModelMaterial, translation: [f32; 3]) -> Self {
        Self {
            mesh,
            material,
            translation,
            scale: [1.0, 1.0, 1.0],
            yaw_radians: 0.0,
        }
    }

    pub const fn scaled(mut self, scale: [f32; 3]) -> Self {
        self.scale = scale;
        self
    }

    pub const fn yawed(mut self, yaw_radians: f32) -> Self {
        self.yaw_radians = yaw_radians;
        self
    }
}

#[derive(Clone, Copy)]
pub enum BlockModel {
    Default,
    Parts(&'static [BlockModelPart]),
    PartsOnly(&'static [BlockModelPart]),
}

#[derive(Clone, Copy)]
pub(crate) struct ColorSpec {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

pub(crate) const fn rgb(r: f32, g: f32, b: f32) -> ColorSpec {
    ColorSpec { r, g, b, a: 1.0 }
}

pub(crate) const fn rgba(r: f32, g: f32, b: f32, a: f32) -> ColorSpec {
    ColorSpec { r, g, b, a }
}

impl ColorSpec {
    fn color(self) -> Color {
        Color::srgba(self.r, self.g, self.b, self.a)
    }
}

impl BlockDefinition {
    pub(crate) const fn scene(
        kind: BlockKind,
        name_key: &'static str,
        short_name_key: &'static str,
        color: ColorSpec,
        slot_color: ColorSpec,
    ) -> Self {
        Self::new(
            kind,
            name_key,
            short_name_key,
            color,
            slot_color,
            BlockClass::Scene,
            Some(PersistentLayer::Puzzle),
        )
    }

    pub(crate) const fn factory(
        kind: BlockKind,
        name_key: &'static str,
        short_name_key: &'static str,
        color: ColorSpec,
        slot_color: ColorSpec,
    ) -> Self {
        Self::new(
            kind,
            name_key,
            short_name_key,
            color,
            slot_color,
            BlockClass::Factory,
            Some(PersistentLayer::SolutionFactory),
        )
    }

    pub(crate) const fn material(
        kind: BlockKind,
        name_key: &'static str,
        short_name_key: &'static str,
        color: ColorSpec,
        slot_color: ColorSpec,
    ) -> Self {
        Self::new(
            kind,
            name_key,
            short_name_key,
            color,
            slot_color,
            BlockClass::Material,
            None,
        )
    }

    pub(crate) const fn marker(
        kind: BlockKind,
        name_key: &'static str,
        short_name_key: &'static str,
        color: ColorSpec,
        slot_color: ColorSpec,
    ) -> Self {
        Self::new(
            kind,
            name_key,
            short_name_key,
            color,
            slot_color,
            BlockClass::Virtual,
            None,
        )
    }

    pub(crate) const fn puzzle_system(
        kind: BlockKind,
        name_key: &'static str,
        short_name_key: &'static str,
        color: ColorSpec,
        slot_color: ColorSpec,
    ) -> Self {
        Self::new(
            kind,
            name_key,
            short_name_key,
            color,
            slot_color,
            BlockClass::System,
            Some(PersistentLayer::Puzzle),
        )
    }

    const fn new(
        kind: BlockKind,
        name_key: &'static str,
        short_name_key: &'static str,
        color: ColorSpec,
        slot_color: ColorSpec,
        class: BlockClass,
        persistence: Option<PersistentLayer>,
    ) -> Self {
        Self {
            kind,
            name_key,
            short_name_key,
            color,
            slot_color,
            class,
            persistence,
            shape: BlockShape::Cube,
            collision: true,
            transparent: false,
            texture: None,
        }
    }

    pub(crate) const fn node(mut self) -> Self {
        self.shape = BlockShape::Node;
        self
    }

    pub(crate) const fn no_collision(mut self) -> Self {
        self.collision = false;
        self
    }

    pub(crate) const fn transparent(mut self) -> Self {
        self.transparent = true;
        self
    }

    pub(crate) const fn textured(mut self, texture: BlockTexture) -> Self {
        self.texture = Some(texture);
        self
    }

    pub fn color(self) -> Color {
        self.color.color()
    }

    pub fn slot_color(self) -> Color {
        self.slot_color.color()
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

    pub fn texture(self) -> Option<BlockTexture> {
        self.texture
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockData {
    pub kind: BlockKind,
    pub facing: Facing,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum MaterialKind {
    #[default]
    Basic,
    Iron,
    Copper,
}

impl MaterialKind {
    pub const ALL: [Self; 3] = [Self::Basic, Self::Iron, Self::Copper];
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum StampColor {
    #[default]
    Red,
    Green,
    Blue,
    Yellow,
}

impl StampColor {
    pub const ALL: [Self; 4] = [Self::Red, Self::Green, Self::Blue, Self::Yellow];

    pub fn name_key(self) -> &'static str {
        match self {
            Self::Red => "stamp_color.red",
            Self::Green => "stamp_color.green",
            Self::Blue => "stamp_color.blue",
            Self::Yellow => "stamp_color.yellow",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Red => Color::srgb(0.95, 0.12, 0.10),
            Self::Green => Color::srgb(0.20, 0.82, 0.28),
            Self::Blue => Color::srgb(0.18, 0.42, 0.95),
            Self::Yellow => Color::srgb(1.0, 0.84, 0.18),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum BlockKind {
    Platform,
    Grass,
    Stone,
    Dirt,
    Planks,
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
    Stamper,
    Roller,
    Converter,
    TeleportEntrance,
    TeleportExit,
    Goal,
    Material,
    IronMaterial,
    CopperMaterial,
    WeldPoint,
    BlockerHead,
    DrillHead,
}

impl BlockKind {
    fn block(self) -> &'static (dyn Block + Send + Sync) {
        registry::get(self)
    }

    pub fn layer(self) -> BlockLayer {
        match self {
            BlockKind::Grass => BlockLayer::Scene(SceneBlock::Grass),
            BlockKind::Stone => BlockLayer::Scene(SceneBlock::Stone),
            BlockKind::Dirt => BlockLayer::Scene(SceneBlock::Dirt),
            BlockKind::Planks => BlockLayer::Scene(SceneBlock::Planks),
            BlockKind::Material => BlockLayer::Material(MaterialBlock::Material),
            BlockKind::IronMaterial => BlockLayer::Material(MaterialBlock::IronMaterial),
            BlockKind::CopperMaterial => BlockLayer::Material(MaterialBlock::CopperMaterial),
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
            BlockKind::Generator => BlockLayer::System(SystemBlock::Generator),
            BlockKind::Goal => BlockLayer::System(SystemBlock::Goal),
            BlockKind::Stamper => BlockLayer::System(SystemBlock::Stamper),
            BlockKind::Roller => BlockLayer::System(SystemBlock::Roller),
            BlockKind::Converter => BlockLayer::System(SystemBlock::Converter),
            BlockKind::TeleportEntrance => BlockLayer::System(SystemBlock::TeleportEntrance),
            BlockKind::TeleportExit => BlockLayer::System(SystemBlock::TeleportExit),
            BlockKind::WeldPoint => BlockLayer::Virtual(VirtualBlock::WeldPoint),
            BlockKind::BlockerHead => BlockLayer::Virtual(VirtualBlock::BlockerHead),
            BlockKind::DrillHead => BlockLayer::Virtual(VirtualBlock::DrillHead),
        }
    }

    pub fn definition(self) -> BlockDefinition {
        self.block().definition()
    }

    pub fn name_key(self) -> &'static str {
        self.definition().name_key
    }

    pub fn short_name_key(self) -> &'static str {
        self.definition().short_name_key
    }

    pub fn material(self) -> Color {
        self.definition().color()
    }

    pub fn slot_color(self) -> Color {
        self.definition().slot_color()
    }

    pub fn shape(self) -> BlockShape {
        self.definition().shape()
    }

    pub fn is_transparent(self) -> bool {
        self.definition().is_transparent()
    }

    pub fn is_directional(self) -> bool {
        self.block().is_directional()
    }

    pub fn has_collision(self) -> bool {
        self.definition().collision
    }

    pub fn blocks_laser(self) -> bool {
        self.has_collision() && !self.is_material()
    }

    pub fn is_factory(self) -> bool {
        matches!(self.layer(), BlockLayer::Factory(_))
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

    pub fn is_system_layer(self) -> bool {
        self.is_system_block() || self.is_generated_marker()
    }

    pub fn accepts_material(self) -> bool {
        matches!(self, BlockKind::Goal)
    }

    pub fn shows_material_preview(self) -> bool {
        matches!(self, BlockKind::Generator | BlockKind::Goal)
    }

    pub fn material_shell_scale(self) -> f32 {
        if self.shows_material_preview() {
            1.08
        } else {
            1.0
        }
    }

    pub fn is_editable(self) -> bool {
        registry::is_editable(self)
    }

    pub fn alternate(self) -> Option<Self> {
        self.block().alternate()
    }

    pub fn ui_panel(self) -> Option<UiPanelId> {
        registry::editable(self).and_then(EditableBlock::ui_panel)
    }

    pub fn marker_behavior(self, facing: Facing) -> Option<MarkerBehavior> {
        self.block().marker_behavior(facing)
    }

    pub fn material_source(self, facing: Facing) -> Option<MaterialSource> {
        self.block().material_source(facing)
    }

    pub fn material_kind(self) -> Option<MaterialKind> {
        self.block().material_kind()
    }

    pub fn material_block_kind(material: MaterialKind) -> Option<Self> {
        registry::material_block_kind(material)
    }

    pub fn persistent_layer(self) -> Option<PersistentLayer> {
        self.block().persistent_layer()
    }

    pub fn default_settings(self, pos: IVec3) -> Option<BlockSettings> {
        self.block().default_settings(pos)
    }

    pub fn movement_rule(self, facing: Facing) -> Option<MovementRule> {
        self.block().movement_rule(facing)
    }

    pub fn material_destroyer(self, facing: Facing) -> Option<MaterialDestroyer> {
        self.block().material_destroyer(facing)
    }

    pub fn material_labeler(self, facing: Facing) -> Option<MaterialLabeler> {
        self.block().material_labeler(facing)
    }

    pub fn weld_behavior(self) -> Option<WeldBehavior> {
        self.block().weld_behavior()
    }

    pub fn signal_behavior(self, facing: Facing) -> Option<SignalBehavior> {
        self.block().signal_behavior(facing)
    }

    pub fn render_behavior(self, facing: Facing) -> RenderBehavior {
        self.block().render_behavior(facing)
    }

    pub fn model(self) -> BlockModel {
        self.block().model()
    }
}
