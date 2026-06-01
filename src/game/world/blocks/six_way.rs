use bevy::prelude::*;

use crate::game::world::grid::WorldBlocks;

use super::BlockData;

pub struct SixWayConnectionPlan {
    pub local_offsets: Vec<IVec3>,
}

pub fn six_way_connection_plan(
    data: BlockData,
    pos: IVec3,
    world: &WorldBlocks,
    connects_to: impl Fn(&BlockData, IVec3) -> bool,
) -> SixWayConnectionPlan {
    let local_offsets = six_way_offsets()
        .into_iter()
        .filter_map(|offset| {
            let neighbor = pos + offset;
            world
                .blocks
                .get(&neighbor)
                .or_else(|| world.system_blocks.get(&neighbor))
                .is_some_and(|block| connects_to(block, -offset))
                .then_some(local_connection_offset(data, offset))
        })
        .collect();

    SixWayConnectionPlan { local_offsets }
}

pub fn local_connection_offset(data: BlockData, offset: IVec3) -> IVec3 {
    if data.kind.is_directional() {
        data.facing.inverse_rotate_offset(offset)
    } else {
        offset
    }
}

pub fn six_way_offsets() -> [IVec3; 6] {
    [
        IVec3::X,
        IVec3::NEG_X,
        IVec3::Y,
        IVec3::NEG_Y,
        IVec3::Z,
        IVec3::NEG_Z,
    ]
}
