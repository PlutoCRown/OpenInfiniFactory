use bevy::prelude::*;

use crate::game::blocks::BlockPresent;
use crate::game::blocks::{BlockData, WeldConnectorBehavior, WireConnectorBehavior};
use crate::game::world::grid::WorldBlocks;

/// 面标记贴在方块表面：只平移；`surface_outset` 为相对方块表面再外凸的距离
pub(crate) fn face_mark_transform(normal: IVec3, surface_outset: f32) -> Transform {
    Transform::from_translation(normal.as_vec3().normalize_or_zero() * (0.5 + surface_outset))
}

/// 俯视（+Y 向下看）逆时针 90°：把水平法线转到相邻轴向
pub(crate) fn rotate_y_ccw(normal: IVec3) -> IVec3 {
    IVec3::new(normal.z, normal.y, -normal.x)
}

/// 判断方块是否在指定方向接受焊接连接
pub(super) fn weld_connects_to(block: &BlockData, connector_from_block: IVec3) -> bool {
    match block.kind.render_behavior(block.facing).weld_connector {
        Some(WeldConnectorBehavior::AllSides) => true,
        Some(WeldConnectorBehavior::Offset(offset)) => connector_from_block == offset,
        None => false,
    }
}

/// 查邻居方块（含系统层）是否接受焊接
pub(super) fn weld_neighbor_connects_to(
    world: &WorldBlocks,
    neighbor: IVec3,
    connector_from_block: IVec3,
) -> bool {
    if let Some(block) = world.system_blocks.get(&neighbor) {
        return weld_connects_to(block, connector_from_block);
    }

    world
        .blocks
        .get(&neighbor)
        .is_some_and(|block| weld_connects_to(block, connector_from_block))
}

/// 把世界方向偏移转成方块局部连接偏移
pub(super) fn local_connector_offset(data: BlockData, offset: IVec3) -> IVec3 {
    if data.kind.is_directional() {
        data.facing.inverse_rotate_offset(offset)
    } else {
        offset
    }
}

/// 判断方块是否在指定方向接受电线连接
pub(super) fn wire_connects_to(block: &BlockData, wire_from_block: IVec3) -> bool {
    match block.kind.render_behavior(block.facing).wire_connector {
        Some(WireConnectorBehavior::Wire) => true,
        Some(WireConnectorBehavior::Device { blocked_offset }) => wire_from_block != blocked_offset,
        None => false,
    }
}

/// 信号/连接器六向偏移
pub(crate) fn signal_neighbor_offsets() -> [IVec3; 6] {
    [
        IVec3::X,
        IVec3::NEG_X,
        IVec3::Y,
        IVec3::NEG_Y,
        IVec3::Z,
        IVec3::NEG_Z,
    ]
}
