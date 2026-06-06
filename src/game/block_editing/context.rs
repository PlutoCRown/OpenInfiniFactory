use bevy::prelude::*;

use crate::game::state::SolutionState;
use crate::game::world::grid::WorldBlocks;

use super::action::BlockPanelAction;
use super::dropdown::{BlockPanelDropdown, OpenBlockPanelDropdown};

pub struct BlockEditContext<'a> {
    pub pos: IVec3,
    pub world: &'a mut WorldBlocks,
    solution_state: &'a mut SolutionState,
    open_dropdown: &'a mut OpenBlockPanelDropdown,
}

impl<'a> BlockEditContext<'a> {
    pub fn new(
        pos: IVec3,
        world: &'a mut WorldBlocks,
        solution_state: &'a mut SolutionState,
        open_dropdown: &'a mut OpenBlockPanelDropdown,
    ) -> Self {
        Self {
            pos,
            world,
            solution_state,
            open_dropdown,
        }
    }

    pub fn toggle_dropdown(&mut self, dropdown: BlockPanelDropdown) {
        self.open_dropdown.0 = (self.open_dropdown.0 != Some(dropdown)).then_some(dropdown);
    }

    pub fn close_dropdown(&mut self) {
        self.open_dropdown.0 = None;
    }

    pub fn mark_dirty(&mut self) {
        self.solution_state.dirty = true;
    }
}

pub fn edit_labeler(ctx: &mut BlockEditContext, action: BlockPanelAction) {
    let mut settings = ctx.world.labeler_settings(ctx.pos);
    match action {
        BlockPanelAction::ToggleColorDropdown => {
            ctx.toggle_dropdown(BlockPanelDropdown::LabelerColor);
            return;
        }
        BlockPanelAction::SetColor(color) => {
            settings.color = color;
            ctx.close_dropdown();
        }
        _ => return,
    }
    ctx.world.set_labeler_settings(ctx.pos, settings);
    ctx.mark_dirty();
}

pub fn edit_teleport(ctx: &mut BlockEditContext, action: BlockPanelAction) {
    match action {
        BlockPanelAction::ToggleTeleportPairDropdown => {
            ctx.toggle_dropdown(BlockPanelDropdown::TeleportPair);
        }
        BlockPanelAction::SetTeleportPair(pair) => {
            let mut settings = ctx.world.teleport_settings(ctx.pos);
            settings.pair = pair;
            ctx.world.set_teleport_settings(ctx.pos, settings);
            ctx.close_dropdown();
            ctx.mark_dirty();
        }
        _ => {}
    }
}
