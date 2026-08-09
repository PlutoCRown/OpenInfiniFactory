//! 工厂块阴影代理：高模 NotShadowCaster，低模用 ShadowProxyMaterial 只投影

/// 本阶段启用阴影代理的工厂种类
pub fn uses_shadow_proxy(kind: crate::game::blocks::BlockKind) -> bool {
    matches!(
        kind,
        crate::game::blocks::BlockKind::Conveyor
            | crate::game::blocks::BlockKind::ReverseConveyor
            | crate::game::blocks::BlockKind::Drill
    )
}
