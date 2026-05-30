mod registry;

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
mod piston;
mod planks;
mod reverse_conveyor;
mod roller;
mod rotator;
mod solid;
mod stamper;
mod stone;
mod teleport_entrance;
mod teleport_exit;
mod weld_point;
mod welder;
mod wire;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub use self::registry::{assert_registry_consistent, ALL_BLOCKS, EDIT_BLOCKS, PLAY_BLOCKS};
use crate::game::ui::UiPanelId;
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

    fn ui_panel(&self) -> Option<UiPanelId> {
        None
    }
}

pub trait SceneBlock: Block {}
pub trait FactoryBlock: Block {}
pub trait MaterialBlock: Block {}
pub trait SystemBlock: Block {}
pub trait EditableBlock: Block {}

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
    system_role: SystemBlockRole,
    persistence: Option<PersistentLayer>,
    shape: BlockShape,
    collision: bool,
    transparent: bool,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum BlockClass {
    Scene,
    Factory,
    Material,
    System,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum SystemBlockRole {
    None,
    GeneratedMarker,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum BlockShape {
    Cube,
    Node,
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
    pub goal_topper: bool,
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
    Large,
    Medium,
    Small,
    Plate,
    RodX,
    RodY,
    RodZ,
    PistonBody,
    PistonHead,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum ModelMaterial {
    Frame,
    DarkFrame,
    Belt,
    BeltStripe,
    Welding,
    Wire,
    Signal,
    Power,
    Piston,
    Wood,
    Lift,
    Rotation,
    Drill,
    Laser,
    System,
    SystemAccent,
    Goal,
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
    Asset {
        path: &'static str,
        fallback: &'static [BlockModelPart],
    },
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
            SystemBlockRole::None,
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
            SystemBlockRole::None,
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
            SystemBlockRole::None,
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
            BlockClass::System,
            SystemBlockRole::GeneratedMarker,
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
            SystemBlockRole::None,
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
        system_role: SystemBlockRole,
        persistence: Option<PersistentLayer>,
    ) -> Self {
        Self {
            kind,
            name_key,
            short_name_key,
            color,
            slot_color,
            class,
            system_role,
            persistence,
            shape: BlockShape::Cube,
            collision: true,
            transparent: false,
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

    pub fn name_key(self) -> &'static str {
        match self {
            Self::Basic => "material.basic",
            Self::Iron => "material.iron",
            Self::Copper => "material.copper",
        }
    }
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
    Solid,
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
    Piston,
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
        self.definition().class() == BlockClass::Factory
    }

    pub fn is_scene(self) -> bool {
        self.definition().class() == BlockClass::Scene
    }

    pub fn is_material(self) -> bool {
        self.definition().class() == BlockClass::Material
    }

    pub fn is_generated_marker(self) -> bool {
        self.definition().system_role == SystemBlockRole::GeneratedMarker
    }

    pub fn is_system_layer(self) -> bool {
        if self.is_generated_marker() {
            return true;
        }

        matches!(
            self,
            BlockKind::Generator
                | BlockKind::Goal
                | BlockKind::Stamper
                | BlockKind::Roller
                | BlockKind::Converter
                | BlockKind::TeleportEntrance
                | BlockKind::TeleportExit
        )
    }

    pub fn is_teleport(self) -> bool {
        matches!(self, BlockKind::TeleportEntrance | BlockKind::TeleportExit)
    }

    pub fn accepts_material(self) -> bool {
        matches!(self, BlockKind::Goal)
    }

    pub fn is_editable(self) -> bool {
        registry::is_editable(self)
    }

    pub fn alternate(self) -> Option<Self> {
        self.block().alternate()
    }

    pub fn ui_panel(self) -> Option<UiPanelId> {
        self.block().ui_panel()
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
