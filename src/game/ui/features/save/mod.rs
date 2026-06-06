mod actions;
pub mod types;

use bevy::prelude::*;

pub use actions::{
    confirm_dialog_actions, save_list_actions, text_prompt_actions,
};
pub(crate) use actions::text_prompt_input;
pub use types::*;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SaveListRenderState::default())
            .insert_resource(ConfirmDialogState::default())
            .insert_resource(TextPromptState::default())
            .add_observer(save_list_actions)
            .add_observer(confirm_dialog_actions)
            .add_observer(text_prompt_actions);
    }
}
