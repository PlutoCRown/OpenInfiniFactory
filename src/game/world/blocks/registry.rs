use super::{
    blocker::BLOCKER, blocker_head::BLOCKER_HEAD, conveyor::CONVEYOR,
    counter_rotator::COUNTER_ROTATOR, detector::DETECTOR, dirt::DIRT,
    down_welder::DOWN_WELDER, drill::DRILL, drill_head::DRILL_HEAD, generator::GENERATOR,
    glass::GLASS, goal::GOAL, grass::GRASS, laser::LASER, lifter::LIFTER,
    material::MATERIAL, piston::PISTON, planks::PLANKS, reverse_conveyor::REVERSE_CONVEYOR,
    rotator::ROTATOR, solid::SOLID, stone::STONE, welder::WELDER, weld_point::WELD_POINT,
    wire::WIRE, Block, BlockKind,
};

pub const EDIT_BLOCKS: [BlockKind; 7] = [
    BlockKind::Grass,
    BlockKind::Stone,
    BlockKind::Dirt,
    BlockKind::Planks,
    BlockKind::Glass,
    BlockKind::Generator,
    BlockKind::Goal,
];

pub const PLAY_BLOCKS: [BlockKind; 14] = [
    BlockKind::Solid,
    BlockKind::Welder,
    BlockKind::DownWelder,
    BlockKind::Conveyor,
    BlockKind::ReverseConveyor,
    BlockKind::Detector,
    BlockKind::Wire,
    BlockKind::Piston,
    BlockKind::Lifter,
    BlockKind::Rotator,
    BlockKind::CounterRotator,
    BlockKind::Blocker,
    BlockKind::Drill,
    BlockKind::Laser,
];

pub const ALL_BLOCKS: [BlockKind; 25] = [
    BlockKind::Grass,
    BlockKind::Stone,
    BlockKind::Dirt,
    BlockKind::Planks,
    BlockKind::Glass,
    BlockKind::Generator,
    BlockKind::Goal,
    BlockKind::Solid,
    BlockKind::Welder,
    BlockKind::DownWelder,
    BlockKind::Conveyor,
    BlockKind::ReverseConveyor,
    BlockKind::Detector,
    BlockKind::Wire,
    BlockKind::Piston,
    BlockKind::Lifter,
    BlockKind::Rotator,
    BlockKind::CounterRotator,
    BlockKind::Blocker,
    BlockKind::Drill,
    BlockKind::Laser,
    BlockKind::Material,
    BlockKind::WeldPoint,
    BlockKind::BlockerHead,
    BlockKind::DrillHead,
];

pub static BLOCK_REGISTRY: [&'static (dyn Block + Send + Sync); 25] = [
    &GRASS,
    &STONE,
    &DIRT,
    &PLANKS,
    &GLASS,
    &GENERATOR,
    &GOAL,
    &SOLID,
    &WELDER,
    &DOWN_WELDER,
    &CONVEYOR,
    &REVERSE_CONVEYOR,
    &DETECTOR,
    &WIRE,
    &PISTON,
    &LIFTER,
    &ROTATOR,
    &COUNTER_ROTATOR,
    &BLOCKER,
    &DRILL,
    &LASER,
    &MATERIAL,
    &WELD_POINT,
    &BLOCKER_HEAD,
    &DRILL_HEAD,
];

pub fn get(kind: BlockKind) -> &'static (dyn Block + Send + Sync) {
    BLOCK_REGISTRY
        .iter()
        .copied()
        .find(|block| block.id() == kind)
        .expect("every BlockKind must be registered")
}
