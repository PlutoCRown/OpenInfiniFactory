mod registry;

mod actions;
mod blocker;
mod blocker_head;
mod catalog;
pub(crate) mod converter;
mod conveyor;
mod counter_rotator;
mod detector;
mod down_detector;
mod down_welder;
mod drill;
mod drill_head;
pub(crate) mod generator;
pub(crate) mod goal;
pub(crate) mod labeler;
mod laser;
mod lifter;
pub(crate) mod panel_layout;
pub(crate) mod panel_systems;
mod platform;
mod pusher;
mod reverse_conveyor;
mod roller;
mod rotator;
mod six_way;
mod stamper;
mod switch;
pub(crate) mod teleport_entrance;
mod teleport_exit;
pub(crate) mod ui_components;
mod weld_point;
mod welder;
mod wire;

use bevy::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub use self::registry::{
    all_blocks, assert_registry_consistent, block_render_assets, edit_blocks, PLAY_BLOCKS,
};
use crate::game::state::SolutionState;
use crate::game::ui::{BlockEditAction, BlockPanelDropdown, OpenBlockPanelDropdown, UiPanelId};
pub use crate::game::world::direction::Facing;
use crate::game::world::grid::WorldBlocks;

pub(crate) use self::converter::{
    converter_settings, set_converter_settings, ConverterMode, ConverterSettings,
};
pub(crate) use self::generator::{generator_settings, set_generator_settings, GeneratorSettings};
pub(crate) use self::goal::{goal_settings, set_goal_settings, GoalSettings};
pub(crate) use self::labeler::{color as labeler_color, set_color as set_labeler_color};
pub(crate) use self::roller::RollerSettings;
pub use self::six_way::{local_connection_offset, six_way_connection_plan, six_way_offsets};
pub(crate) use self::stamper::StamperSettings;
pub(crate) use self::teleport_entrance::{
    set_teleport_settings, teleport_menu_actions, teleport_rename_input, teleport_settings,
    TeleportSettings,
};
pub use self::wire::wire_connector_render_plan;
use crate::shared::i18n::I18n;
pub(crate) use actions::block_edit_actions;
pub(crate) use panel_systems::{
    update_block_panel_dropdowns_ui, update_converter_ui, update_generator_ui, update_labeler_ui,
    update_teleport_ui,
};

pub const BLOCK_SIZE: f32 = 1.0;
pub const DEFAULT_GENERATOR_PERIOD: u64 = 3;

pub(crate) fn spawn_block_panels(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    generator::ui::spawn_panel(root, i18n);
    goal::ui::spawn_panel(root, i18n);
    labeler::ui::spawn_panel(root, i18n);
    converter::ui::spawn_panel(root, i18n);
    teleport_entrance::ui::spawn_panel(root, i18n);
}

pub(crate) fn spawn_block_dropdown_layers(root: &mut ChildSpawnerCommands, i18n: &I18n) {
    generator::ui::spawn_dropdown_layers(root);
    goal::ui::spawn_dropdown_layers(root);
    labeler::ui::spawn_dropdown_layers(root, i18n);
    converter::ui::spawn_dropdown_layers(root);
    teleport_entrance::ui::spawn_dropdown_layers(root);
}

pub trait RenderableBlock {
    fn render_definition(&self) -> BlockDefinition;
    fn render_behavior(&self, facing: Facing) -> RenderBehavior;
    fn render_model(&self) -> BlockModel;
}

impl<T: Block + ?Sized> RenderableBlock for T {
    fn render_definition(&self) -> BlockDefinition {
        self.definition()
    }

    fn render_behavior(&self, facing: Facing) -> RenderBehavior {
        Block::render_behavior(self, facing)
    }

    fn render_model(&self) -> BlockModel {
        self.model()
    }
}

pub trait Block: RenderableBlock + Send + Sync {
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

    fn default_state(&self, _pos: IVec3, _world: &WorldBlocks) -> Option<SerializedBlockState> {
        None
    }

    fn normalize_state(
        &self,
        _state: &SerializedBlockState,
        _pos: IVec3,
    ) -> Option<SerializedBlockState> {
        None
    }

    fn on_removed(&self, _pos: IVec3, _world: &mut WorldBlocks) {}

    fn movement_rule(&self, _facing: Facing) -> Option<MovementRule> {
        None
    }

    fn factory_connection_blocker(&self, _facing: Facing) -> Option<IVec3> {
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

    fn render_assets(&self) -> BlockRenderAssets {
        BlockRenderAssets::default()
    }

    fn model(&self) -> BlockModel {
        BlockModel::Default
    }

    fn alternate(&self) -> Option<BlockKind> {
        None
    }
}

pub trait SerializableBlockState:
    Clone + Default + DeserializeOwned + PartialEq + Serialize + Send + Sync + 'static
{
    const BLOCK_KINDS: &'static [BlockKind];

    fn default_for(_pos: IVec3, _world: &WorldBlocks) -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SerializedBlockState {
    payload: serde_json::Value,
}

impl SerializedBlockState {
    pub fn from_state<T: SerializableBlockState>(state: &T) -> Option<Self> {
        serde_json::to_value(state)
            .ok()
            .map(|payload| Self { payload })
    }

    pub fn decode<T: SerializableBlockState>(&self) -> Option<T> {
        serde_json::from_value(self.payload.clone()).ok()
    }

    pub fn debug_signature(&self) -> String {
        ron::ser::to_string(&self.payload).unwrap_or_else(|_| format!("{:?}", self.payload))
    }
}

pub trait EditableBlock: Block {
    fn ui_panel(&self) -> Option<UiPanelId> {
        None
    }

    fn handle_edit_action(&self, _ctx: &mut BlockEditContext, _action: BlockEditAction) {}
}

pub struct BlockEditContext<'a> {
    pub pos: IVec3,
    pub world: &'a mut WorldBlocks,
    solution_state: &'a mut SolutionState,
    open_dropdown: &'a mut OpenBlockPanelDropdown,
}

impl<'a> BlockEditContext<'a> {
    pub fn new(
        pos: IVec3,
        world: &'a mut WorldBlocks,
        solution_state: &'a mut SolutionState,
        open_dropdown: &'a mut OpenBlockPanelDropdown,
    ) -> Self {
        Self {
            pos,
            world,
            solution_state,
            open_dropdown,
        }
    }

    pub fn toggle_dropdown(&mut self, dropdown: BlockPanelDropdown) {
        self.open_dropdown.0 = (self.open_dropdown.0 != Some(dropdown)).then_some(dropdown);
    }

    pub fn close_dropdown(&mut self) {
        self.open_dropdown.0 = None;
    }

    pub fn mark_dirty(&mut self) {
        self.solution_state.dirty = true;
    }
}

pub(super) fn edit_labeler(ctx: &mut BlockEditContext, action: BlockEditAction) {
    match action {
        BlockEditAction::ToggleColorDropdown => {
            ctx.toggle_dropdown(BlockPanelDropdown::LabelerColor);
            return;
        }
        BlockEditAction::SetColor(color) => {
            set_labeler_color(ctx.world, ctx.pos, color);
            ctx.close_dropdown();
        }
        _ => return,
    }
    ctx.mark_dirty();
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
    Switch,
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

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
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

#[derive(Clone, Copy)]
pub enum ModelMeshSpec {
    Cuboid {
        size: [f32; 3],
    },
    CoveredCuboid {
        size: [f32; 3],
    },
    Cylinder {
        radius: f32,
        height: f32,
        resolution: u32,
    },
    Ring {
        outer_radius: f32,
        inner_radius: f32,
        height: f32,
        segments: u32,
    },
    DrillTip {
        radius: f32,
        length: f32,
        segments: u32,
    },
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum ModelMaterial {
    ConveyorBelt,
    DrillTip,
    DarkFrame,
    Belt,
    BeltStripe,
    WeldCore,
    Signal,
    Power,
    Platform,
    PlatformBase,
    BorderedWoodTexture,
    StoneTexture,
    Lift,
    Laser,
    System,
    SystemAccent,
    Goal,
    TeleportIn,
    TeleportOut,
}

#[derive(Clone, Copy)]
pub enum ModelMaterialSpec {
    Srgb {
        color: ColorSpec,
    },
    Emissive {
        color: ColorSpec,
        emissive: ColorSpec,
    },
    Textured {
        color: ColorSpec,
        texture: BlockTexture,
    },
}

#[derive(Clone, Copy, Default)]
pub struct BlockRenderAssets {
    pub meshes: &'static [(ModelMesh, ModelMeshSpec)],
    pub materials: &'static [(ModelMaterial, ModelMaterialSpec)],
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
pub struct BlockRenderSpec {
    pub definition: BlockDefinition,
    pub behavior: RenderBehavior,
    pub model: BlockModel,
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
    pub(crate) fn color(self) -> Color {
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
            BlockClass::System,
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
            Self::System(_) | Self::Virtual(_) => BlockClass::System,
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
    Switch,
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
            BlockKind::Switch => BlockLayer::Factory(FactoryBlock::Switch),
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

    pub fn is_system_layer(self) -> bool {
        matches!(self.layer(), BlockLayer::System(_) | BlockLayer::Virtual(_))
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

    pub fn handle_edit_action(
        self,
        pos: IVec3,
        action: BlockEditAction,
        world: &mut WorldBlocks,
        solution_state: &mut SolutionState,
        open_dropdown: &mut OpenBlockPanelDropdown,
    ) {
        if let Some(block) = registry::editable(self) {
            let mut ctx = BlockEditContext::new(pos, world, solution_state, open_dropdown);
            block.handle_edit_action(&mut ctx, action);
        }
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

    pub fn default_state(self, pos: IVec3, world: &WorldBlocks) -> Option<SerializedBlockState> {
        self.block().default_state(pos, world)
    }

    pub fn normalize_state(
        self,
        state: &SerializedBlockState,
        pos: IVec3,
    ) -> Option<SerializedBlockState> {
        self.block().normalize_state(state, pos)
    }

    pub fn on_removed(self, pos: IVec3, world: &mut WorldBlocks) {
        self.block().on_removed(pos, world);
    }

    pub fn movement_rule(self, facing: Facing) -> Option<MovementRule> {
        self.block().movement_rule(facing)
    }

    pub fn factory_connection_blocker(self, facing: Facing) -> Option<IVec3> {
        self.block().factory_connection_blocker(facing)
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

    pub fn render_spec(self, facing: Facing) -> BlockRenderSpec {
        let block = self.block();
        BlockRenderSpec {
            definition: RenderableBlock::render_definition(block),
            behavior: RenderableBlock::render_behavior(block, facing),
            model: RenderableBlock::render_model(block),
        }
    }
}
