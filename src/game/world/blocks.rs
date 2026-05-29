mod registry;

mod blocker;
mod blocker_head;
mod conveyor;
mod counter_rotator;
mod detector;
mod dirt;
mod down_welder;
mod drill;
mod drill_head;
mod generator;
mod glass;
mod goal;
mod grass;
mod laser;
mod lifter;
mod material;
mod piston;
mod planks;
mod reverse_conveyor;
mod rotator;
mod solid;
mod stone;
mod welder;
mod weld_point;
mod wire;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub use crate::game::world::direction::Facing;
pub use self::registry::{ALL_BLOCKS, EDIT_BLOCKS, PLAY_BLOCKS};

pub const BLOCK_SIZE: f32 = 1.0;
pub const DEFAULT_GENERATOR_PERIOD: u64 = 3;

pub trait Block: Send + Sync {
    fn id(&self) -> BlockKind;
    fn definition(&self) -> BlockDefinition;

    fn marker_behavior(&self, _facing: Facing) -> Option<MarkerBehavior> {
        None
    }

    fn material_source(&self, _facing: Facing) -> Option<MaterialSource> {
        None
    }

    fn material_mover(&self, _facing: Facing) -> Option<MaterialMover> {
        None
    }

    fn material_destroyer(&self, _facing: Facing) -> Option<MaterialDestroyer> {
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
}

pub trait SceneBlock: Block {}
pub trait FactoryBlock: Block {}
pub trait SystemBlock: Block {}

#[derive(Clone, Copy)]
pub struct BlockDefinition {
    pub kind: BlockKind,
    pub name_key: &'static str,
    pub short_name_key: &'static str,
    color: ColorSpec,
    slot_color: ColorSpec,
    class: BlockClass,
    system_role: SystemBlockRole,
    shape: BlockShape,
    directional: bool,
    collision: bool,
    transparent: bool,
    alternate: Option<BlockKind>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum BlockClass {
    Scene,
    Factory,
    System,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum SystemBlockRole {
    None,
    Material,
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
    Generator { output: IVec3 },
}

#[derive(Clone, Copy)]
pub enum MaterialMover {
    Conveyor { source: IVec3, offset: IVec3 },
    Lifter,
    Rotator { clockwise: bool },
    Piston { source: IVec3, offset: IVec3 },
}

#[derive(Clone, Copy)]
pub enum MaterialDestroyer {
    Drill { target: IVec3 },
    AdjacentDrillHead,
    Laser { direction: IVec3, range: i32 },
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
            BlockClass::System,
            SystemBlockRole::Material,
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
    ) -> Self {
        Self {
            kind,
            name_key,
            short_name_key,
            color,
            slot_color,
            class,
            system_role,
            shape: BlockShape::Cube,
            directional: false,
            collision: true,
            transparent: false,
            alternate: None,
        }
    }

    pub(crate) const fn directional(mut self) -> Self {
        self.directional = true;
        self
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

    pub(crate) const fn alternate(mut self, alternate: BlockKind) -> Self {
        self.alternate = Some(alternate);
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
}

impl MaterialKind {
    pub const ALL: [Self; 1] = [Self::Basic];

    pub fn name_key(self) -> &'static str {
        match self {
            Self::Basic => "material.basic",
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
    Glass,
    Generator,
    Welder,
    DownWelder,
    Conveyor,
    ReverseConveyor,
    Detector,
    Wire,
    Piston,
    Lifter,
    Rotator,
    CounterRotator,
    Blocker,
    Drill,
    Laser,
    Goal,
    Material,
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
        self.definition().directional
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
        self.definition().system_role == SystemBlockRole::Material
    }

    pub fn is_generated_marker(self) -> bool {
        self.definition().system_role == SystemBlockRole::GeneratedMarker
    }

    pub fn alternate(self) -> Option<Self> {
        self.definition().alternate
    }

    pub fn marker_behavior(self, facing: Facing) -> Option<MarkerBehavior> {
        self.block().marker_behavior(facing)
    }

    pub fn material_source(self, facing: Facing) -> Option<MaterialSource> {
        self.block().material_source(facing)
    }

    pub fn material_mover(self, facing: Facing) -> Option<MaterialMover> {
        self.block().material_mover(facing)
    }

    pub fn material_destroyer(self, facing: Facing) -> Option<MaterialDestroyer> {
        self.block().material_destroyer(facing)
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
}
