use crate::game::blocks::BlockKind;

use super::SaveFormatError;

/// 将方块种类编码为 u8；Material/Stamp 仅写种类标记，字符串 id 由 write_block 追加
pub fn encode_kind(kind: BlockKind) -> u8 {
    match kind {
        BlockKind::Platform => 0,
        // 场景方块 v3 走字符串段；此处仅兼容旧 u8 表里的四种
        BlockKind::Scene(id) => match oif_sim::blocks::scene_catalog().string_id(id) {
            Some("grass") => 1,
            Some("stone") => 2,
            Some("dirt") => 3,
            Some("planks") => 4,
            other => panic!(
                "encode_kind: scene block {:?} must use string id section",
                other
            ),
        },
        BlockKind::Generator => 5,
        BlockKind::Welder => 6,
        BlockKind::DownWelder => 7,
        BlockKind::Conveyor => 8,
        BlockKind::ReverseConveyor => 9,
        BlockKind::Detector => 10,
        BlockKind::DownDetector => 11,
        BlockKind::Wire => 12,
        BlockKind::Pusher => 13,
        BlockKind::Lifter => 14,
        BlockKind::Rotator => 15,
        BlockKind::CounterRotator => 16,
        BlockKind::Blocker => 17,
        BlockKind::Drill => 18,
        BlockKind::Laser => 19,
        BlockKind::Mirror => 20,
        BlockKind::VerticalMirror => 21,
        BlockKind::Splitter => 22,
        BlockKind::SuctionCup => 34,
        BlockKind::Sign => 39,
        BlockKind::Stamper => 23,
        BlockKind::Roller => 24,
        BlockKind::Converter => 25,
        BlockKind::TeleportEntrance => 26,
        BlockKind::TeleportExit => 27,
        BlockKind::Goal => 28,
        // v5：种类标记后跟字符串 id；旧档无字符串时由 decode_kind 映射固定材料
        BlockKind::Material(_) => 29,
        BlockKind::Stamp(_) => 38,
        BlockKind::WeldPoint => 32,
        BlockKind::DrillHead => 33,
        BlockKind::RollerBody => 36,
        BlockKind::StamperBody => 37,
    }
}

/// 解码非 Material/Stamp 的固定种类；v4 及以下材料/印花映射到默认字符串 id
pub fn decode_kind(id: u8) -> Result<BlockKind, SaveFormatError> {
    Ok(match id {
        0 => BlockKind::Platform,
        1 => BlockKind::scene("grass"),
        2 => BlockKind::scene("stone"),
        3 => BlockKind::scene("dirt"),
        4 => BlockKind::scene("planks"),
        5 => BlockKind::Generator,
        6 => BlockKind::Welder,
        7 => BlockKind::DownWelder,
        8 => BlockKind::Conveyor,
        9 => BlockKind::ReverseConveyor,
        10 => BlockKind::Detector,
        11 => BlockKind::DownDetector,
        12 => BlockKind::Wire,
        13 => BlockKind::Pusher,
        14 => BlockKind::Lifter,
        15 => BlockKind::Rotator,
        16 => BlockKind::CounterRotator,
        17 => BlockKind::Blocker,
        18 => BlockKind::Drill,
        19 => BlockKind::Laser,
        20 => BlockKind::Mirror,
        21 => BlockKind::VerticalMirror,
        22 => BlockKind::Splitter,
        23 => BlockKind::Stamper,
        24 => BlockKind::Roller,
        25 => BlockKind::Converter,
        26 => BlockKind::TeleportEntrance,
        27 => BlockKind::TeleportExit,
        28 => BlockKind::Goal,
        29 => BlockKind::material("basic"),
        30 => BlockKind::material("iron"),
        31 => BlockKind::material("copper"),
        32 => BlockKind::WeldPoint,
        33 => BlockKind::DrillHead,
        34 => BlockKind::SuctionCup,
        35 => BlockKind::material("glass_material"),
        36 => BlockKind::RollerBody,
        37 => BlockKind::StamperBody,
        38 => BlockKind::stamp("red"),
        39 => BlockKind::Sign,
        _ => return Err(SaveFormatError::UnknownBlockKind(id)),
    })
}
