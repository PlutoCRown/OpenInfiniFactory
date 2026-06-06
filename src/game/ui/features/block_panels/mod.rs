mod actions;
mod update;

use bevy::prelude::*;

use crate::game::block_editing::OpenBlockPanelDropdown;
use crate::game::systems::perf::PerfScope;
use crate::game::ui::core::text_input::InlineTextEditState;

pub use actions::{dispatch_block_panel_actions, emit_block_panel_actions};
pub use update::{update_active_block_panel, update_block_panel_dropdowns};

pub(crate) use actions::inline_text_edit_input;

pub struct BlockPanelsPlugin;

impl Plugin for BlockPanelsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OpenBlockPanelDropdown::default())
            .insert_resource(InlineTextEditState::default())
            .add_observer(emit_block_panel_actions)
            .add_systems(
                Update,
                dispatch_block_panel_actions
                    .after(PerfScope::Input)
                    .before(PerfScope::Menus),
            );
    }
}
