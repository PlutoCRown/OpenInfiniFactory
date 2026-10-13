//! 场景方块运行时目录：字符串 id ↔ SceneBlockId，以及碰撞/connectable 等模拟元数据

use std::sync::{LazyLock, RwLock};

use serde::{Deserialize, Serialize};

use super::catalog_store::{
    CatalogEntry, HasStringId, StringIdCatalog, clone_global, install_global, with_global,
    with_global_mut,
};
use super::{ColorSpec, rgb};

pub use super::catalog_store::leak_str;

/// 找不到资源包时使用的兜底场景方块 string id
pub const FALLBACK_SCENE_STRING_ID: &str = "fallback";

/// 场景方块句柄（会话内稳定；存档存字符串 id，不依赖固定编号）
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
pub struct SceneBlockId(pub u16);

impl From<u16> for SceneBlockId {
    fn from(v: u16) -> Self {
        Self(v)
    }
}

impl From<SceneBlockId> for u16 {
    fn from(id: SceneBlockId) -> Self {
        id.0
    }
}

/// 单种场景方块的模拟侧定义
#[derive(Clone, Debug)]
pub struct SceneBlockDef {
    pub string_id: String,
    /// i18n key，形如 `block.grass`（泄漏为 `'static` 供旧 API）
    pub name_key: &'static str,
    pub short_name_key: &'static str,
    pub description_key: &'static str,
    pub collision: bool,
    pub connectable: [bool; 6],
    /// 是否可朝向（放置时 R 旋转，存档持久化 facing）
    pub directional: bool,
    pub color: ColorSpec,
}

impl HasStringId for SceneBlockDef {
    fn string_id(&self) -> &str {
        &self.string_id
    }
}

impl CatalogEntry for SceneBlockDef {
    const LABEL: &'static str = "scene block";
}

/// 场景方块目录
pub type SceneBlockCatalog = StringIdCatalog<SceneBlockId, SceneBlockDef>;

impl SceneBlockCatalog {
    /// 解析 string id；未知则返回兜底场景方块
    pub fn resolve_id(&self, string_id: &str) -> SceneBlockId {
        self.id_by_string(string_id)
            .unwrap_or_else(|| self.fallback_id())
    }

    /// 目录内兜底场景方块（必存在）
    pub fn fallback_id(&self) -> SceneBlockId {
        self.id_by_string(FALLBACK_SCENE_STRING_ID)
            .or_else(|| (!self.is_empty()).then_some(SceneBlockId(0)))
            .expect("scene catalog must contain fallback")
    }
}

static CATALOG: LazyLock<RwLock<SceneBlockCatalog>> =
    LazyLock::new(|| RwLock::new(SceneBlockCatalog::new()));

/// 确保目录里有醒目的兜底场景方块
fn ensure_fallback_entry(catalog: &mut SceneBlockCatalog) {
    if catalog.id_by_string(FALLBACK_SCENE_STRING_ID).is_some() {
        return;
    }
    catalog
        .register(SceneBlockDef {
            string_id: FALLBACK_SCENE_STRING_ID.to_string(),
            name_key: leak_str("block.fallback"),
            short_name_key: leak_str("short.fallback"),
            description_key: leak_str("desc.fallback"),
            collision: true,
            connectable: [true; 6],
            directional: false,
            // UI 无贴图时的色块提示（洋红）
            color: rgb(1.0, 0.0, 1.0),
        })
        .expect("fallback scene id unique");
}

/// 安装/替换全局场景目录；保证兜底方块存在
pub fn install_scene_catalog(mut catalog: SceneBlockCatalog) {
    ensure_fallback_entry(&mut catalog);
    install_global(&CATALOG, catalog);
}

/// 读取当前目录快照
pub fn scene_catalog() -> SceneBlockCatalog {
    ensure_fallback_scene_catalog();
    clone_global(&CATALOG)
}

/// 当前兜底场景方块 id
pub fn fallback_scene_id() -> SceneBlockId {
    ensure_fallback_scene_catalog();
    with_global(&CATALOG, |c| c.fallback_id())
}

/// 按 string id 解析；未知 → 兜底
pub fn resolve_scene_id(string_id: &str) -> SceneBlockId {
    ensure_fallback_scene_catalog();
    with_global(&CATALOG, |c| c.resolve_id(string_id))
}

/// 按 id 查定义；未知 id → 兜底定义
pub fn scene_def(id: SceneBlockId) -> SceneBlockDef {
    ensure_fallback_scene_catalog();
    with_global(&CATALOG, |catalog| {
        catalog.get(id).cloned().unwrap_or_else(|| {
            catalog
                .get(catalog.fallback_id())
                .cloned()
                .expect("fallback def")
        })
    })
}

/// 确保兜底场景方块已注册
pub fn ensure_fallback_scene_catalog() {
    if with_global(&CATALOG, |c| {
        c.id_by_string(FALLBACK_SCENE_STRING_ID).is_some()
    }) {
        return;
    }
    with_global_mut(&CATALOG, ensure_fallback_entry);
}

/// 用资源元数据验证场景连接面、朝向和模拟锚定行为
#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{BlockData, BlockKind};
    use crate::session::SimSession;
    use crate::simulation::structure_state::StructureState;
    use crate::world::Facing;
    use crate::world::grid::WorldBlocks;
    use glam::IVec3;
    use std::collections::HashSet;

    /// 草不能固定工厂，斜坡按旋转后的可连接面固定，编辑重新计算锚定
    #[test]
    fn factory_anchors_follow_scene_metadata_and_orientation() {
        ensure_fallback_scene_catalog();
        let scene_ids = with_global_mut(&CATALOG, |catalog| {
            // 保留旧测试使用的 0..=6 兜底句柄，新增定义不改变并行测试的场景语义
            let fallback = catalog.get(catalog.fallback_id()).unwrap().clone();
            while catalog.len() <= 6 {
                let mut def = fallback.clone();
                def.string_id = format!("test_legacy_scene_{}", catalog.len());
                catalog.register(def).unwrap();
            }
            [
                (
                    "test_short_grass",
                    include_str!("../../../../assets/scene_blocks/short_grass/meta.json"),
                ),
                (
                    "test_quartz_slope",
                    include_str!("../../../../assets/scene_blocks/quartz_slope/meta.json"),
                ),
                (
                    "test_basic",
                    include_str!("../../../../assets/scene_blocks/basic/meta.json"),
                ),
            ]
            .map(|(string_id, metadata)| {
                let metadata: serde_json::Value = serde_json::from_str(metadata).unwrap();
                let mut def = fallback.clone();
                def.string_id = string_id.to_string();
                def.collision = metadata["collision"].as_bool().unwrap_or(true);
                def.connectable = serde_json::from_value(metadata["connectable"].clone()).unwrap();
                def.directional = metadata["directional"].as_bool().unwrap_or(false);
                catalog.register(def).unwrap()
            })
        });
        let normals = [
            IVec3::X,
            IVec3::NEG_X,
            IVec3::Y,
            IVec3::NEG_Y,
            IVec3::Z,
            IVec3::NEG_Z,
        ];
        let facings = [Facing::North, Facing::East, Facing::South, Facing::West];
        let expected_faces = [
            [[false; 6]; 4],
            [
                [true, true, false, true, false, true],
                [true, false, false, true, true, true],
                [true, true, false, true, true, false],
                [false, true, false, true, true, true],
            ],
            [[true; 6]; 4],
        ];
        let scene_pos = IVec3::new(0, 3, 0);
        for (fixture, scene_id) in scene_ids.into_iter().enumerate() {
            for (facing_index, facing) in facings.into_iter().enumerate() {
                for (face_index, normal) in normals.into_iter().enumerate() {
                    let mut world = WorldBlocks::default();
                    world.insert(
                        scene_pos,
                        BlockData::new(BlockKind::Scene(scene_id), facing),
                    );
                    let factory_pos = scene_pos + normal;
                    world.insert(
                        factory_pos,
                        BlockData::new(BlockKind::Platform, Facing::North),
                    );
                    let mut structures = StructureState::default();
                    structures.rebuild_for_simulation(&world);
                    let structure = structures
                        .get(structures.id_at(factory_pos).unwrap())
                        .unwrap();
                    assert_eq!(
                        structure.is_anchored(),
                        expected_faces[fixture][facing_index][face_index],
                        "fixture={fixture}, facing={facing:?}, normal={normal:?}"
                    );
                }
            }
        }

        let mut session = SimSession::new();
        let factory_pos = scene_pos + IVec3::X;
        session.world.insert(
            scene_pos,
            BlockData::new(BlockKind::Scene(scene_ids[0]), Facing::North),
        );
        let factory = BlockData::new(BlockKind::Platform, Facing::North);
        session.world.insert(factory_pos, factory);
        let factory_id = session.world.blocks[&factory_pos].id;
        session.begin_simulation();
        session.simulate_next_turn();
        assert_eq!(
            session.world.blocks[&(factory_pos + IVec3::NEG_Y)].id,
            factory_id
        );

        let mut world = WorldBlocks::default();
        let factory_pos = scene_pos + IVec3::Z;
        world.insert(
            scene_pos,
            BlockData::new(BlockKind::Scene(scene_ids[1]), Facing::North),
        );
        world.insert(factory_pos, factory);
        let mut structures = StructureState::default();
        structures.rebuild_for_simulation(&world);
        assert!(
            !structures
                .get(structures.id_at(factory_pos).unwrap())
                .unwrap()
                .is_anchored()
        );
        world.insert(
            scene_pos,
            BlockData::new(BlockKind::Scene(scene_ids[1]), Facing::South),
        );
        structures.apply_factory_edit(&world, &HashSet::from([scene_pos]));
        assert!(
            structures
                .get(structures.id_at(factory_pos).unwrap())
                .unwrap()
                .is_anchored()
        );
        world.remove(&scene_pos);
        structures.apply_factory_edit(&world, &HashSet::from([scene_pos]));
        assert!(
            !structures
                .get(structures.id_at(factory_pos).unwrap())
                .unwrap()
                .is_anchored()
        );
    }
}
