use oif_sim::BlockKind;

use super::SaveFormatError;

/// 将方块种类编码为 u8；Material/Stamp 仅写种类标记，字符串 id 由 write_block 追加
pub fn encode_kind(kind: BlockKind) -> u8 {
    match kind {
        BlockKind::Platform => 0,
        BlockKind::Scene(_) => {
            panic!("encode_kind: scene blocks must use the string id section")
        }
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
        // 种类标记后跟字符串 id
        BlockKind::Material(_) => 29,
        BlockKind::Stamp(_) => 38,
        BlockKind::WeldPoint => 32,
        BlockKind::DrillHead => 33,
        BlockKind::RollerBody => 36,
        BlockKind::StamperBody => 37,
    }
}

/// 解码非 Material/Stamp 的固定种类；旧档场景/材料数字一律落到兜底
pub fn decode_kind(id: u8) -> Result<BlockKind, SaveFormatError> {
    Ok(match id {
        0 => BlockKind::Platform,
        // 旧场景数字 id → 兜底
        1 | 2 | 3 | 4 => BlockKind::Scene(oif_sim::blocks::fallback_scene_id()),
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
        // 旧材料数字 id → 兜底（v5 会再读字符串覆盖）
        29 | 30 | 31 | 35 => BlockKind::Material(oif_sim::blocks::fallback_material_id()),
        32 => BlockKind::WeldPoint,
        33 => BlockKind::DrillHead,
        34 => BlockKind::SuctionCup,
        36 => BlockKind::RollerBody,
        37 => BlockKind::StamperBody,
        38 => BlockKind::stamp("red"),
        39 => BlockKind::Sign,
        _ => return Err(SaveFormatError::UnknownBlockKind(id)),
    })
}
