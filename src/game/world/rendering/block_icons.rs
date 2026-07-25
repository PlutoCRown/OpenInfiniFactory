//! 预烘焙方块图标路径（工厂/编辑系统块）；与 bake_scene_icons --factory 共用

use std::path::PathBuf;

use crate::game::blocks::BlockKind;
use crate::shared::platform;

/// 选区工具图标（非 BlockKind）
pub const SELECTION_ICON_RELPATH: &str = "factory_blocks/selection/icon.png";

/// 预烘焙图标相对 `assets/` 的路径；场景/材料/印花走各自 registry，不在此列
pub fn baked_block_icon_relpath(kind: BlockKind) -> Option<&'static str> {
    match kind {
        BlockKind::Platform => Some("factory_blocks/platform/icon.png"),
        BlockKind::Welder => Some("factory_blocks/welder/icon.png"),
        BlockKind::DownWelder => Some("factory_blocks/welder/icon_down.png"),
        BlockKind::Conveyor => Some("factory_blocks/conveyor/icon.png"),
        BlockKind::ReverseConveyor => Some("factory_blocks/conveyor/icon_reverse.png"),
        BlockKind::Detector => Some("factory_blocks/detector/icon.png"),
        BlockKind::DownDetector => Some("factory_blocks/detector/icon_down.png"),
        BlockKind::Wire => Some("factory_blocks/wire/icon.png"),
        BlockKind::Pusher => Some("factory_blocks/pusher/icon.png"),
        BlockKind::Lifter => Some("factory_blocks/lifter/icon.png"),
        BlockKind::Rotator => Some("factory_blocks/rotator/icon.png"),
        BlockKind::CounterRotator => Some("factory_blocks/counter_rotator/icon.png"),
        BlockKind::Blocker => Some("factory_blocks/blocker/icon.png"),
        BlockKind::Drill => Some("factory_blocks/drill/icon.png"),
        BlockKind::Laser => Some("factory_blocks/laser/icon.png"),
        BlockKind::Mirror => Some("factory_blocks/mirror/icon.png"),
        BlockKind::VerticalMirror => Some("factory_blocks/vertical_mirror/icon.png"),
        BlockKind::Splitter => Some("factory_blocks/splitter/icon.png"),
        BlockKind::SuctionCup => Some("factory_blocks/suction_cup/icon.png"),
        // 无 factory_blocks 目录的编辑/系统块
        BlockKind::Sign => Some("block_icons/sign.png"),
        BlockKind::Generator => Some("block_icons/generator.png"),
        BlockKind::Goal => Some("block_icons/goal.png"),
        BlockKind::Converter => Some("block_icons/converter.png"),
        BlockKind::Stamper => Some("block_icons/stamper.png"),
        BlockKind::Roller => Some("block_icons/roller.png"),
        BlockKind::TeleportEntrance => Some("block_icons/teleport_entrance.png"),
        BlockKind::TeleportExit => Some("block_icons/teleport_exit.png"),
        BlockKind::Scene(_)
        | BlockKind::Material(_)
        | BlockKind::Stamp(_)
        | BlockKind::WeldPoint
        | BlockKind::DrillHead
        | BlockKind::RollerBody
        | BlockKind::StamperBody => None,
    }
}

/// 预烘焙图标绝对路径
pub fn baked_block_icon_path(kind: BlockKind) -> Option<PathBuf> {
    baked_block_icon_relpath(kind).map(|rel| PathBuf::from(platform::asset_path()).join(rel))
}

/// 选区工具图标绝对路径
pub fn selection_icon_path() -> PathBuf {
    PathBuf::from(platform::asset_path()).join(SELECTION_ICON_RELPATH)
}

/// `--only` 匹配用短 id（目录名或 block_icons 文件名）
pub fn baked_block_icon_only_id(kind: BlockKind) -> Option<&'static str> {
    match kind {
        BlockKind::Platform => Some("platform"),
        BlockKind::Welder => Some("welder"),
        BlockKind::DownWelder => Some("down_welder"),
        BlockKind::Conveyor => Some("conveyor"),
        BlockKind::ReverseConveyor => Some("reverse_conveyor"),
        BlockKind::Detector => Some("detector"),
        BlockKind::DownDetector => Some("down_detector"),
        BlockKind::Wire => Some("wire"),
        BlockKind::Pusher => Some("pusher"),
        BlockKind::Lifter => Some("lifter"),
        BlockKind::Rotator => Some("rotator"),
        BlockKind::CounterRotator => Some("counter_rotator"),
        BlockKind::Blocker => Some("blocker"),
        BlockKind::Drill => Some("drill"),
        BlockKind::Laser => Some("laser"),
        BlockKind::Mirror => Some("mirror"),
        BlockKind::VerticalMirror => Some("vertical_mirror"),
        BlockKind::Splitter => Some("splitter"),
        BlockKind::SuctionCup => Some("suction_cup"),
        BlockKind::Sign => Some("sign"),
        BlockKind::Generator => Some("generator"),
        BlockKind::Goal => Some("goal"),
        BlockKind::Converter => Some("converter"),
        BlockKind::Stamper => Some("stamper"),
        BlockKind::Roller => Some("roller"),
        BlockKind::TeleportEntrance => Some("teleport_entrance"),
        BlockKind::TeleportExit => Some("teleport_exit"),
        _ => None,
    }
}

/// 需要预烘焙图标的工厂/系统方块（热键栏用）
pub fn bakeable_block_icon_kinds() -> Vec<BlockKind> {
    use crate::game::blocks::{PLAY_BLOCKS, edit_blocks};

    let mut kinds = Vec::new();
    for kind in edit_blocks().into_iter().chain(PLAY_BLOCKS) {
        if baked_block_icon_relpath(kind).is_none() {
            continue;
        }
        if !kinds.contains(&kind) {
            kinds.push(kind);
        }
    }
    kinds
}
