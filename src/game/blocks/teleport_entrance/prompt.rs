use bevy::prelude::IVec3;

use crate::game::state::SolutionState;
use crate::game::ui::access::{i18n, ui};
use crate::game::ui::core::text_prompt::{TextPromptProps, TextPromptResult};
use crate::game::world::grid::WorldBlocks;

pub fn open_teleport_rename_prompt(pos: IVec3, current_name: String) {
    let spec = TextPromptProps {
        title: i18n.t("teleport.prompt.rename"),
        default_value: current_name,
        save_text: i18n.t("button.confirm"),
        cancel_text: i18n.t("button.cancel"),
    };
    ui.open_text_prompt_then(spec, move |result, world| {
        let TextPromptResult::Saved(requested) = result else {
            return;
        };
        let trimmed = requested.trim();
        if trimmed.is_empty() {
            return;
        }
        let name = trimmed.chars().take(24).collect::<String>();
        let mut world_blocks = world.resource_mut::<WorldBlocks>();
        if !world_blocks.system_blocks.contains_key(&pos) {
            return;
        }
        let mut settings = world_blocks.teleport_settings(pos);
        if settings.name == name {
            return;
        }
        settings.name = name;
        world_blocks.set_teleport_settings(pos, settings);
        world.resource_mut::<SolutionState>().dirty = true;
    });
}
