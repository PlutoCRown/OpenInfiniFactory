mod actions;
mod confirm;
mod prompt;
pub mod types;
mod update;
mod view;

use bevy::prelude::*;

pub use actions::{dispatch_save_list_actions, emit_save_list_actions, text_prompt_input};
pub use confirm::{
    open_save_puzzle_confirm, open_save_puzzle_confirm_before_exit, EXTRA_SAVE_AS,
};
pub use prompt::open_save_as_new_puzzle_prompt;
pub use types::*;
pub use update::update_save_list_ui;

use crate::game::systems::perf::PerfScope;
use crate::game::ui::access::UiAccessScope;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SaveListRenderState::default())
            .add_observer(emit_save_list_actions)
            .add_systems(
                Update,
                (
                    dispatch_save_list_actions
                        .in_set(UiAccessScope)
                        .after(PerfScope::Input)
                        .before(PerfScope::Menus),
                    update_save_list_ui
                        .in_set(UiAccessScope)
                        .after(PerfScope::Animation)
                        .before(PerfScope::Ui),
                ),
            );
    }
}
