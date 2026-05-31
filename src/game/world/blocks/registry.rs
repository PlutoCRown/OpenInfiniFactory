use super::{
    blocker::BLOCKER, blocker_head::BLOCKER_HEAD, catalog::BasicBlockRegistration,
    converter::CONVERTER, conveyor::CONVEYOR, counter_rotator::COUNTER_ROTATOR, detector::DETECTOR,
    down_detector::DOWN_DETECTOR, down_welder::DOWN_WELDER, drill::DRILL, drill_head::DRILL_HEAD,
    generator::GENERATOR, goal::GOAL, laser::LASER, lifter::LIFTER, platform::PLATFORM,
    pusher::PUSHER, reverse_conveyor::REVERSE_CONVEYOR, roller::ROLLER, rotator::ROTATOR,
    stamper::STAMPER, switch::SWITCH, teleport_entrance::TELEPORT_ENTRANCE,
    teleport_exit::TELEPORT_EXIT, weld_point::WELD_POINT, welder::WELDER, wire::WIRE, Block,
    BlockKind, EditableBlock, MaterialKind,
};

pub const BUILTIN_EDIT_BLOCKS: [BlockKind; 7] = [
    BlockKind::Generator,
    BlockKind::Goal,
    BlockKind::Stamper,
    BlockKind::Roller,
    BlockKind::Converter,
    BlockKind::TeleportEntrance,
    BlockKind::TeleportExit,
];

pub const PLAY_BLOCKS: [BlockKind; 16] = [
    BlockKind::Platform,
    BlockKind::Welder,
    BlockKind::DownWelder,
    BlockKind::Conveyor,
    BlockKind::ReverseConveyor,
    BlockKind::Detector,
    BlockKind::DownDetector,
    BlockKind::Wire,
    BlockKind::Switch,
    BlockKind::Pusher,
    BlockKind::Lifter,
    BlockKind::Rotator,
    BlockKind::CounterRotator,
    BlockKind::Blocker,
    BlockKind::Drill,
    BlockKind::Laser,
];

pub const BUILTIN_BLOCKS: [BlockKind; 26] = [
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
    BlockKind::Switch,
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
    BlockKind::WeldPoint,
    BlockKind::BlockerHead,
    BlockKind::DrillHead,
];

pub static BUILTIN_BLOCK_REGISTRY: [&'static (dyn Block + Send + Sync); 26] = [
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
    &SWITCH,
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
    &WELD_POINT,
    &BLOCKER_HEAD,
    &DRILL_HEAD,
];

const BUILTIN_EDITABLE_REGISTRY: [&'static (dyn EditableBlock + Send + Sync); 7] = [
    &GENERATOR,
    &GOAL,
    &STAMPER,
    &ROLLER,
    &CONVERTER,
    &TELEPORT_ENTRANCE,
    &TELEPORT_EXIT,
];

pub fn edit_blocks() -> Vec<BlockKind> {
    let mut blocks: Vec<_> = basic_block_registrations()
        .filter_map(|registration| registration.editable.then_some(registration.kind))
        .chain(BUILTIN_EDIT_BLOCKS)
        .collect();
    sort_blocks(&mut blocks);
    blocks
}

pub fn all_blocks() -> Vec<BlockKind> {
    let mut blocks: Vec<_> = basic_block_registrations()
        .map(|registration| registration.kind)
        .chain(BUILTIN_BLOCKS)
        .collect();
    sort_blocks(&mut blocks);
    blocks
}

pub fn get(kind: BlockKind) -> &'static (dyn Block + Send + Sync) {
    basic_block_registrations()
        .find_map(|registration| (registration.kind == kind).then_some(registration.block))
        .or_else(|| {
            BUILTIN_BLOCK_REGISTRY
                .iter()
                .copied()
                .find(|block| block.id() == kind)
        })
        .expect("every BlockKind must be registered")
}

pub fn is_editable(kind: BlockKind) -> bool {
    basic_block_registrations()
        .any(|registration| registration.kind == kind && registration.editable)
        || BUILTIN_EDITABLE_REGISTRY
            .iter()
            .any(|block| block.id() == kind)
}

pub fn editable(kind: BlockKind) -> Option<&'static (dyn EditableBlock + Send + Sync)> {
    basic_block_registrations()
        .find_map(|registration| {
            (registration.kind == kind)
                .then_some(registration.editable_block)
                .flatten()
        })
        .or_else(|| {
            BUILTIN_EDITABLE_REGISTRY
                .iter()
                .copied()
                .find(|block| block.id() == kind)
        })
}

pub fn material_block_kind(material: MaterialKind) -> Option<BlockKind> {
    basic_block_registrations()
        .find_map(|registration| {
            (registration.block.material_kind() == Some(material)).then_some(registration.kind)
        })
        .or(match material {
            MaterialKind::Basic => Some(BlockKind::Material),
            MaterialKind::Iron => Some(BlockKind::IronMaterial),
            MaterialKind::Copper => Some(BlockKind::CopperMaterial),
        })
}

pub fn assert_registry_consistent() {
    for registration in basic_block_registrations() {
        let definition = registration.block.definition();
        debug_assert_eq!(definition.kind, registration.kind);
        debug_assert_eq!(definition.kind, registration.block.id());
        debug_assert_eq!(definition.class(), registration.kind.layer().class());
        debug_assert_eq!(registration.editable, registration.editable_block.is_some());
    }

    for block in BUILTIN_BLOCK_REGISTRY {
        let definition = block.definition();
        debug_assert_eq!(definition.kind, block.id());
        debug_assert_eq!(definition.class(), block.id().layer().class());
    }

    for kind in edit_blocks() {
        debug_assert!(is_editable(kind));
        debug_assert!(editable(kind).is_some());
    }
}

fn basic_block_registrations() -> impl Iterator<Item = &'static BasicBlockRegistration> {
    inventory::iter::<BasicBlockRegistration>.into_iter()
}

fn sort_blocks(blocks: &mut [BlockKind]) {
    blocks.sort_by_key(|kind| block_order(*kind));
}

fn block_order(kind: BlockKind) -> usize {
    match kind {
        BlockKind::Grass => 0,
        BlockKind::Stone => 1,
        BlockKind::Dirt => 2,
        BlockKind::Planks => 3,
        BlockKind::Generator => 4,
        BlockKind::Goal => 5,
        BlockKind::Platform => 6,
        BlockKind::Welder => 7,
        BlockKind::DownWelder => 8,
        BlockKind::Conveyor => 9,
        BlockKind::ReverseConveyor => 10,
        BlockKind::Detector => 11,
        BlockKind::DownDetector => 12,
        BlockKind::Wire => 13,
        BlockKind::Switch => 14,
        BlockKind::Pusher => 15,
        BlockKind::Lifter => 16,
        BlockKind::Rotator => 17,
        BlockKind::CounterRotator => 18,
        BlockKind::Blocker => 19,
        BlockKind::Drill => 20,
        BlockKind::Laser => 21,
        BlockKind::Stamper => 22,
        BlockKind::Roller => 23,
        BlockKind::Converter => 24,
        BlockKind::TeleportEntrance => 25,
        BlockKind::TeleportExit => 26,
        BlockKind::Material => 27,
        BlockKind::IronMaterial => 28,
        BlockKind::CopperMaterial => 29,
        BlockKind::WeldPoint => 30,
        BlockKind::BlockerHead => 31,
        BlockKind::DrillHead => 32,
    }
}
