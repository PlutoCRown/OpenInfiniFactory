use super::traits::PlaceableBlock;
use super::{Block, BlockKind, EditableBlock};
use oif_sim::blocks::scene_catalog;

pub fn edit_blocks() -> Vec<BlockKind> {
    let mut blocks: Vec<_> = registrations()
        .filter_map(|registration| registration.editable.then_some(registration.kind))
        .collect();
    blocks.extend(scene_kinds());
    sort_blocks(&mut blocks);
    blocks
}

pub fn all_blocks() -> Vec<BlockKind> {
    let mut blocks: Vec<_> = registrations()
        .map(|registration| registration.kind)
        .collect();
    blocks.extend(scene_kinds());
    sort_blocks(&mut blocks);
    blocks
}

fn scene_kinds() -> Vec<BlockKind> {
    scene_catalog()
        .iter()
        .map(|(id, _)| BlockKind::Scene(id))
        .collect()
}

pub const PLAY_BLOCKS: [BlockKind; 20] = [
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
    BlockKind::Mirror,
    BlockKind::VerticalMirror,
    BlockKind::Splitter,
    BlockKind::SuctionCup,
    BlockKind::Sign,
];

pub fn get(kind: BlockKind) -> &'static (dyn Block + Send + Sync) {
    registrations()
        .find_map(|registration| (registration.kind == kind).then_some(registration.block))
        .expect("every BlockKind must be registered")
}

pub fn save_stores_facing(kind: BlockKind) -> bool {
    oif_sim::blocks::save_stores_facing(kind)
}

pub fn is_editable(kind: BlockKind) -> bool {
    // 场景块不走 inventory 注册，但编辑热键栏可放置
    if matches!(kind, BlockKind::Scene(_)) {
        return true;
    }
    registrations().any(|registration| registration.kind == kind && registration.editable)
}

pub fn editable(kind: BlockKind) -> Option<&'static (dyn EditableBlock + Send + Sync)> {
    registrations().find_map(|registration| {
        (registration.kind == kind)
            .then_some(registration.editable_block)
            .flatten()
    })
}

pub fn placeable(kind: BlockKind) -> Option<&'static (dyn PlaceableBlock + Send + Sync)> {
    registrations().find_map(|registration| {
        (registration.kind == kind)
            .then_some(registration.placeable_block)
            .flatten()
    })
}

pub fn assert_registry_consistent() {
    oif_sim::blocks::assert_registry_consistent();
    for registration in registrations() {
        let definition = registration.block.definition();
        debug_assert_eq!(definition.kind, registration.kind);
        debug_assert_eq!(definition.kind, registration.block.id());
        debug_assert_eq!(definition.class(), registration.kind.layer().class());
        debug_assert_eq!(registration.editable, registration.editable_block.is_some());
        debug_assert_eq!(
            registration.editable || registration.play_palette,
            registration.placeable_block.is_some()
        );

        if registration.play_palette {
            debug_assert!(PLAY_BLOCKS.contains(&registration.kind));
        }
    }

    for kind in edit_blocks() {
        debug_assert!(is_editable(kind));
        // 场景块没有 EditableBlock 实现（无设置面板），只校验可编辑标记
        if matches!(kind, BlockKind::Scene(_)) {
            continue;
        }
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
        BlockKind::Scene(id) => id.0 as usize,
        BlockKind::Generator => 100,
        BlockKind::Goal => 101,
        BlockKind::Platform => 102,
        BlockKind::Welder => 103,
        BlockKind::DownWelder => 104,
        BlockKind::Conveyor => 105,
        BlockKind::ReverseConveyor => 106,
        BlockKind::Detector => 107,
        BlockKind::DownDetector => 108,
        BlockKind::Wire => 109,
        BlockKind::Pusher => 110,
        BlockKind::Lifter => 111,
        BlockKind::Rotator => 112,
        BlockKind::CounterRotator => 113,
        BlockKind::Blocker => 114,
        BlockKind::Drill => 115,
        BlockKind::Laser => 116,
        BlockKind::Mirror => 117,
        BlockKind::VerticalMirror => 118,
        BlockKind::Splitter => 119,
        BlockKind::SuctionCup => 120,
        BlockKind::Sign => 121,
        BlockKind::Stamper => 122,
        BlockKind::Roller => 123,
        BlockKind::Converter => 124,
        BlockKind::TeleportEntrance => 125,
        BlockKind::TeleportExit => 126,
        BlockKind::Material => 127,
        BlockKind::IronMaterial => 128,
        BlockKind::CopperMaterial => 129,
        BlockKind::GlassMaterial => 130,
        BlockKind::StampMaterial => 135,
        BlockKind::WeldPoint => 131,
        BlockKind::DrillHead => 132,
        BlockKind::RollerBody => 133,
        BlockKind::StamperBody => 134,
    }
}
