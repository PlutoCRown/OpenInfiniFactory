use super::{Block, BlockKind};

/// 按 kind 查模拟侧注册表
pub fn get(kind: BlockKind) -> &'static (dyn Block + Send + Sync) {
    registrations()
        .find_map(|registration| (registration.kind == kind).then_some(registration.block))
        .expect("every BlockKind must be registered")
}

/// 存档是否需要持久化朝向
pub fn save_stores_facing(kind: BlockKind) -> bool {
    match kind {
        BlockKind::Scene(id) => super::scene_def(id).directional,
        BlockKind::Material(id) => super::material_def(id).directional,
        BlockKind::Stamp(_) => false,
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
