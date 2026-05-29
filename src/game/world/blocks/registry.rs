use super::{
    blocker::BLOCKER, blocker_head::BLOCKER_HEAD, conveyor::CONVEYOR, converter::CONVERTER,
    copper_material::COPPER_MATERIAL, counter_rotator::COUNTER_ROTATOR, detector::DETECTOR,
    dirt::DIRT, down_detector::DOWN_DETECTOR, down_welder::DOWN_WELDER, drill::DRILL,
    drill_head::DRILL_HEAD, generator::GENERATOR, glass::GLASS, goal::GOAL, grass::GRASS,
    iron_material::IRON_MATERIAL, laser::LASER, lifter::LIFTER, material::MATERIAL,
    piston::PISTON, planks::PLANKS, reverse_conveyor::REVERSE_CONVEYOR, roller::ROLLER,
    rotator::ROTATOR, solid::SOLID, stamper::STAMPER, stone::STONE,
    teleport_entrance::TELEPORT_ENTRANCE, teleport_exit::TELEPORT_EXIT, welder::WELDER,
    weld_point::WELD_POINT, wire::WIRE, Block, BlockKind,
};

pub const EDIT_BLOCKS: [BlockKind; 12] = [
    BlockKind::Grass,
    BlockKind::Stone,
    BlockKind::Dirt,
    BlockKind::Planks,
    BlockKind::Glass,
    BlockKind::Generator,
    BlockKind::Goal,
    BlockKind::Stamper,
    BlockKind::Roller,
    BlockKind::Converter,
    BlockKind::TeleportEntrance,
    BlockKind::TeleportExit,
];

pub const EDITABLE_BLOCKS: [BlockKind; 12] = EDIT_BLOCKS;

pub const PLAY_BLOCKS: [BlockKind; 15] = [
    BlockKind::Solid,
    BlockKind::Welder,
    BlockKind::DownWelder,
    BlockKind::Conveyor,
    BlockKind::ReverseConveyor,
    BlockKind::Detector,
    BlockKind::DownDetector,
    BlockKind::Wire,
    BlockKind::Piston,
    BlockKind::Lifter,
    BlockKind::Rotator,
    BlockKind::CounterRotator,
    BlockKind::Blocker,
    BlockKind::Drill,
    BlockKind::Laser,
];

pub const ALL_BLOCKS: [BlockKind; 33] = [
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
    BlockKind::DownDetector,
    BlockKind::Wire,
    BlockKind::Piston,
    BlockKind::Lifter,
    BlockKind::Rotator,
    BlockKind::CounterRotator,
    BlockKind::Blocker,
    BlockKind::Drill,
    BlockKind::Laser,
    BlockKind::Stamper,
    BlockKind::Roller,
    BlockKind::Converter,
    BlockKind::TeleportEntrance,
    BlockKind::TeleportExit,
    BlockKind::Material,
    BlockKind::IronMaterial,
    BlockKind::CopperMaterial,
    BlockKind::WeldPoint,
    BlockKind::BlockerHead,
    BlockKind::DrillHead,
];

pub static BLOCK_REGISTRY: [&'static (dyn Block + Send + Sync); 33] = [
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
    &DOWN_DETECTOR,
    &WIRE,
    &PISTON,
    &LIFTER,
    &ROTATOR,
    &COUNTER_ROTATOR,
    &BLOCKER,
    &DRILL,
    &LASER,
    &STAMPER,
    &ROLLER,
    &CONVERTER,
    &TELEPORT_ENTRANCE,
    &TELEPORT_EXIT,
    &MATERIAL,
    &IRON_MATERIAL,
    &COPPER_MATERIAL,
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

pub fn is_editable(kind: BlockKind) -> bool {
    EDITABLE_BLOCKS.contains(&kind)
}
