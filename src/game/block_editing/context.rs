use bevy::prelude::*;

use crate::game::state::SolutionState;
use crate::game::world::grid::WorldBlocks;

use super::panel_state::OpenBlockPanelDropdown;

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

    pub fn toggle_dropdown(&mut self, panel: crate::game::state::UiPanelId, slot: u8) {
        self.open_dropdown.toggle(panel, slot);
    }

    pub fn close_dropdown(&mut self) {
        self.open_dropdown.close();
    }

    pub fn mark_dirty(&mut self) {
        self.solution_state.dirty = true;
    }
}
