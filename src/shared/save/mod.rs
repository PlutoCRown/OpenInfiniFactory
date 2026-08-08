//! 存档读写：Puzzle / Free / Solution 槽位与世界序列化

use bevy::prelude::*;
use oif_sim::blocks::{BlockData, BlockKind, PersistentLayer};
use oif_sim::world::grid::{BlockSettings, MaterialFace, WorldBlocks};
use serde::{Deserialize, Serialize};

use crate::shared::persistent_storage;
use crate::shared::save_format::{
    self, BLOCKS_FILE, COVER_FILE, META_FILE, SKYBOX_FILE, SaveBlocksData, SavedBlock,
};

include!("types.rs");

/// 新建谜题时写入的默认天空盒（模板缺失时的兜底）
const DEFAULT_SKYBOX_PNG: &[u8] = include_bytes!("../../../assets/skybox.png");

/// 默认新存档模板（嵌入；桌面也可改 assets/save_templates/default/）
const TEMPLATE_META_JSON: &str = include_str!("../../../assets/save_templates/default/meta.json");
const TEMPLATE_BLOCKS_BIN: &[u8] =
    include_bytes!("../../../assets/save_templates/default/blocks.bin");
const TEMPLATE_SKYBOX_PNG: &[u8] =
    include_bytes!("../../../assets/save_templates/default/skybox.png");

const SAVE_VERSION: u32 = 1;

include!("ops.rs");
include!("file.rs");
include!("capture.rs");
include!("list.rs");
