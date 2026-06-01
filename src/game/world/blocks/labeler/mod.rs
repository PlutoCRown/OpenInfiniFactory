pub(crate) mod ui;

use bevy::prelude::*;

use crate::game::world::blocks::{
    roller, stamper, BlockKind, RollerSettings, StampColor, StamperSettings,
};
use crate::game::world::grid::WorldBlocks;

pub(crate) fn color(world: &WorldBlocks, pos: IVec3) -> StampColor {
    if world
        .system_blocks
        .get(&pos)
        .is_some_and(|block| block.kind == BlockKind::Roller)
    {
        roller::roller_settings(world, pos).color
    } else {
        stamper::stamper_settings(world, pos).color
    }
}

pub(crate) fn set_color(world: &mut WorldBlocks, pos: IVec3, color: StampColor) {
    if world
        .system_blocks
        .get(&pos)
        .is_some_and(|block| block.kind == BlockKind::Roller)
    {
        roller::set_roller_settings(world, pos, RollerSettings { color });
    } else {
        stamper::set_stamper_settings(world, pos, StamperSettings { color });
    }
}
