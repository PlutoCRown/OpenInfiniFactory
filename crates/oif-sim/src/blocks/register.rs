use super::{Block, BlockKind};

/// 模拟侧方块注册项（仅 Meta + Behavior）
pub struct BlockRegistration {
    pub kind: BlockKind,
    pub block: &'static (dyn Block + Send + Sync),
}

inventory::collect!(BlockRegistration);

/// 各方块 mod.rs 底部一行注册；忽略 editable/play（表现侧在主 crate）
macro_rules! register_block {
    ($block:expr, $kind:expr $(, $($rest:tt)*)?) => {
        inventory::submit! {
            $crate::blocks::register::BlockRegistration {
                kind: $kind,
                block: &$block,
            }
        }
    };
}
