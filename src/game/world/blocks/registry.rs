use super::{
    blocker::BLOCKER, blocker_head::BLOCKER_HEAD, converter::CONVERTER, conveyor::CONVEYOR,
    copper_material::COPPER_MATERIAL, counter_rotator::COUNTER_ROTATOR, detector::DETECTOR,
    dirt::DIRT, down_detector::DOWN_DETECTOR, down_welder::DOWN_WELDER, drill::DRILL,
    drill_head::DRILL_HEAD, generator::GENERATOR, goal::GOAL, grass::GRASS,
    iron_material::IRON_MATERIAL, laser::LASER, lifter::LIFTER, material::MATERIAL, pusher::PUSHER,
    planks::PLANKS, reverse_conveyor::REVERSE_CONVEYOR, roller::ROLLER, rotator::ROTATOR,
    platform::PLATFORM, stamper::STAMPER, stone::STONE, teleport_entrance::TELEPORT_ENTRANCE,
    teleport_exit::TELEPORT_EXIT, weld_point::WELD_POINT, welder::WELDER, wire::WIRE, Block,
    BlockKind, EditableBlock, FactoryBlock, MaterialBlock, MaterialKind, SceneBlock, SystemBlock,
};

pub const EDIT_BLOCKS: [BlockKind; 11] = [
    BlockKind::Grass,
    BlockKind::Stone,
    BlockKind::Dirt,
    BlockKind::Planks,
    BlockKind::Generator,
    BlockKind::Goal,
    BlockKind::Stamper,
    BlockKind::Roller,
    BlockKind::Converter,
    BlockKind::TeleportEntrance,
    BlockKind::TeleportExit,
];

pub const EDITABLE_BLOCKS: [BlockKind; 11] = EDIT_BLOCKS;

pub const PLAY_BLOCKS: [BlockKind; 15] = [
    BlockKind::Platform,
    BlockKind::Welder,
    BlockKind::DownWelder,
    BlockKind::Conveyor,
    BlockKind::ReverseConveyor,
    BlockKind::Detector,
    BlockKind::DownDetector,
    BlockKind::Wire,
    BlockKind::Pusher,
    BlockKind::Lifter,
    BlockKind::Rotator,
    BlockKind::CounterRotator,
    BlockKind::Blocker,
    BlockKind::Drill,
    BlockKind::Laser,
];

pub const ALL_BLOCKS: [BlockKind; 32] = [
    BlockKind::Grass,
    BlockKind::Stone,
    BlockKind::Dirt,
    BlockKind::Planks,
    BlockKind::Generator,
    BlockKind::Goal,
    BlockKind::Platform,
    BlockKind::Welder,
    BlockKind::DownWelder,
    BlockKind::Conveyor,
    BlockKind::ReverseConveyor,
    BlockKind::Detector,
    BlockKind::DownDetector,
    BlockKind::Wire,
    BlockKind::Pusher,
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

pub static BLOCK_REGISTRY: [&'static (dyn Block + Send + Sync); 32] = [
    &GRASS,
    &STONE,
    &DIRT,
    &PLANKS,
    &GENERATOR,
    &GOAL,
    &PLATFORM,
    &WELDER,
    &DOWN_WELDER,
    &CONVEYOR,
    &REVERSE_CONVEYOR,
    &DETECTOR,
    &DOWN_DETECTOR,
    &WIRE,
    &PUSHER,
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

const SCENE_REGISTRY: [&'static (dyn SceneBlock + Send + Sync); 4] =
    [&GRASS, &STONE, &DIRT, &PLANKS];
const FACTORY_REGISTRY: [&'static (dyn FactoryBlock + Send + Sync); 15] = [
    &PLATFORM,
    &WELDER,
    &DOWN_WELDER,
    &CONVEYOR,
    &REVERSE_CONVEYOR,
    &DETECTOR,
    &DOWN_DETECTOR,
    &WIRE,
    &PUSHER,
    &LIFTER,
    &ROTATOR,
    &COUNTER_ROTATOR,
    &BLOCKER,
    &DRILL,
    &LASER,
];
const MATERIAL_REGISTRY: [&'static (dyn MaterialBlock + Send + Sync); 3] =
    [&MATERIAL, &IRON_MATERIAL, &COPPER_MATERIAL];
const SYSTEM_REGISTRY: [&'static (dyn SystemBlock + Send + Sync); 10] = [
    &GENERATOR,
    &GOAL,
    &STAMPER,
    &ROLLER,
    &CONVERTER,
    &TELEPORT_ENTRANCE,
    &TELEPORT_EXIT,
    &WELD_POINT,
    &BLOCKER_HEAD,
    &DRILL_HEAD,
];
const EDITABLE_REGISTRY: [&'static (dyn EditableBlock + Send + Sync); 11] = [
    &GRASS,
    &STONE,
    &DIRT,
    &PLANKS,
    &GENERATOR,
    &GOAL,
    &STAMPER,
    &ROLLER,
    &CONVERTER,
    &TELEPORT_ENTRANCE,
    &TELEPORT_EXIT,
];

pub fn get(kind: BlockKind) -> &'static (dyn Block + Send + Sync) {
    BLOCK_REGISTRY
        .iter()
        .copied()
        .find(|block| block.id() == kind)
        .expect("every BlockKind must be registered")
}

pub fn is_editable(kind: BlockKind) -> bool {
    EDITABLE_REGISTRY.iter().any(|block| block.id() == kind)
}

pub fn material_block_kind(material: MaterialKind) -> Option<BlockKind> {
    MATERIAL_REGISTRY
        .iter()
        .find_map(|block| (block.material_kind() == Some(material)).then_some(block.id()))
}

pub fn assert_registry_consistent() {
    let _ = SCENE_REGISTRY;
    let _ = FACTORY_REGISTRY;
    let _ = SYSTEM_REGISTRY;

    for block in BLOCK_REGISTRY {
        let definition = block.definition();
        debug_assert_eq!(definition.kind, block.id());
    }

    for kind in EDITABLE_BLOCKS {
        debug_assert!(is_editable(kind));
    }
}
