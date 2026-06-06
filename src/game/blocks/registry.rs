use super::{Block, BlockKind, EditableBlock, MaterialKind};

pub fn edit_blocks() -> Vec<BlockKind> {
    let mut blocks: Vec<_> = registrations()
        .filter_map(|registration| registration.editable.then_some(registration.kind))
        .collect();
    sort_blocks(&mut blocks);
    blocks
}

pub fn all_blocks() -> Vec<BlockKind> {
    let mut blocks: Vec<_> = registrations().map(|registration| registration.kind).collect();
    sort_blocks(&mut blocks);
    blocks
}

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

pub fn get(kind: BlockKind) -> &'static (dyn Block + Send + Sync) {
    registrations()
        .find_map(|registration| (registration.kind == kind).then_some(registration.block))
        .expect("every BlockKind must be registered")
}

pub fn is_editable(kind: BlockKind) -> bool {
    registrations()
        .any(|registration| registration.kind == kind && registration.editable)
}

pub fn editable(kind: BlockKind) -> Option<&'static (dyn EditableBlock + Send + Sync)> {
    registrations().find_map(|registration| {
        (registration.kind == kind)
            .then_some(registration.editable_block)
            .flatten()
    })
}

pub fn material_block_kind(material: MaterialKind) -> Option<BlockKind> {
    registrations()
        .find_map(|registration| {
            (registration.block.material_kind() == Some(material)).then_some(registration.kind)
        })
}

pub fn assert_registry_consistent() {
    for registration in registrations() {
        let definition = registration.block.definition();
        debug_assert_eq!(definition.kind, registration.kind);
        debug_assert_eq!(definition.kind, registration.block.id());
        debug_assert_eq!(definition.class(), registration.kind.layer().class());
        debug_assert_eq!(registration.editable, registration.editable_block.is_some());

        if registration.play_palette {
            debug_assert!(PLAY_BLOCKS.contains(&registration.kind));
        }
    }

    for kind in edit_blocks() {
        debug_assert!(is_editable(kind));
        debug_assert!(editable(kind).is_some());
    }

    for kind in PLAY_BLOCKS {
        debug_assert!(registrations().any(|registration| registration.kind == kind));
    }
}

fn registrations() -> impl Iterator<Item = &'static super::register::BlockRegistration> {
    inventory::iter::<super::register::BlockRegistration>.into_iter()
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
        BlockKind::Pusher => 14,
        BlockKind::Lifter => 15,
        BlockKind::Rotator => 16,
        BlockKind::CounterRotator => 17,
        BlockKind::Blocker => 18,
        BlockKind::Drill => 19,
        BlockKind::Laser => 20,
        BlockKind::Stamper => 21,
        BlockKind::Roller => 22,
        BlockKind::Converter => 23,
        BlockKind::TeleportEntrance => 24,
        BlockKind::TeleportExit => 25,
        BlockKind::Material => 26,
        BlockKind::IronMaterial => 27,
        BlockKind::CopperMaterial => 28,
        BlockKind::WeldPoint => 29,
        BlockKind::BlockerHead => 30,
        BlockKind::DrillHead => 31,
    }
}
