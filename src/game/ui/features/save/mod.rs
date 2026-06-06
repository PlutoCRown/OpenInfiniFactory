mod actions;
mod confirm;
mod prompt;
pub mod types;
mod update;
mod view;

use bevy::prelude::*;

pub use actions::{save_list_actions, text_prompt_input};
pub use prompt::{open_save_as_new_puzzle_prompt, SaveTextPromptParams};
pub use types::*;
pub use update::update_save_list_ui;

use crate::game::systems::perf::PerfScope;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SaveListRenderState::default())
            .add_observer(save_list_actions)
            .add_systems(
                Update,
                update_save_list_ui
                    .after(PerfScope::Animation)
                    .before(PerfScope::Ui),
            );
    }
}
