use super::{Block, BlockKind, MaterialKind};

/// 按 kind 查模拟侧注册表
pub fn get(kind: BlockKind) -> &'static (dyn Block + Send + Sync) {
    registrations()
        .find_map(|registration| (registration.kind == kind).then_some(registration.block))
        .expect("every BlockKind must be registered")
}

/// 材料种类对应的方块 kind
pub fn material_block_kind(material: MaterialKind) -> Option<BlockKind> {
    registrations().find_map(|registration| {
        (registration.block.material_kind() == Some(material)).then_some(registration.kind)
    })
}

/// 存档是否需要持久化朝向
pub fn save_stores_facing(kind: BlockKind) -> bool {
    if let BlockKind::Scene(id) = kind {
        return super::scene_def(id).directional;
    }
    match kind {
        BlockKind::Platform | BlockKind::Wire | BlockKind::DownWelder | BlockKind::DownDetector => {
            false
        }
        kind => get(kind).is_directional(),
    }
}

/// 启动时校验注册表一致性
pub fn assert_registry_consistent() {
    for registration in registrations() {
        let definition = registration.block.definition();
        debug_assert_eq!(definition.kind, registration.kind);
        debug_assert_eq!(definition.kind, registration.block.id());
        debug_assert_eq!(definition.class(), registration.kind.layer().class());
    }
}

fn registrations() -> impl Iterator<Item = &'static super::register::BlockRegistration> {
    inventory::iter::<super::register::BlockRegistration>.into_iter()
}
