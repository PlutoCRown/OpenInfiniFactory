use super::traits::PlaceableBlock;
use super::{Block, BlockKind, EditableBlock};

/// 游戏侧方块注册（含表现 / 可放置 / 可编辑）
pub struct BlockRegistration {
    pub kind: BlockKind,
    pub block: &'static (dyn Block + Send + Sync),
    pub editable_block: Option<&'static (dyn EditableBlock + Send + Sync)>,
    pub placeable_block: Option<&'static (dyn PlaceableBlock + Send + Sync)>,
    pub editable: bool,
    pub play_palette: bool,
}

inventory::collect!(BlockRegistration);

/// 各方块 mod.rs 底部一行注册
macro_rules! register_block {
    ($block:expr, $kind:expr, editable: false, play: true $(,)?) => {
        inventory::submit! {
            $crate::game::blocks::register::BlockRegistration {
                kind: $kind,
                block: &$block,
                editable_block: None,
                placeable_block: Some(&$block),
                editable: false,
                play_palette: true,
            }
        }
    };
    ($block:expr, $kind:expr, editable: false $(,)?) => {
        inventory::submit! {
            $crate::game::blocks::register::BlockRegistration {
                kind: $kind,
                block: &$block,
                editable_block: None,
                placeable_block: None,
                editable: false,
                play_palette: false,
            }
        }
    };
    ($block:expr, $kind:expr, editable: true $(, play: $play:expr)? $(,)?) => {
        inventory::submit! {
            $crate::game::blocks::register::BlockRegistration {
                kind: $kind,
                block: &$block,
                editable_block: Some(&$block),
                placeable_block: Some(&$block),
                editable: true,
                play_palette: { register_block!(@play $($play)?) },
            }
        }
    };
    (@play) => { false };
    (@play $play:expr) => { $play };
}
