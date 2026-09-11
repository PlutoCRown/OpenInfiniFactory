//! 编辑模式配置复制粘贴，以及选区工具快捷切换

use bevy::prelude::*;

use crate::game::block_editing::world_refresh::apply_block_settings_edit;
use crate::game::local_player::ClipboardPlayerMut;
use crate::game::session::EditableWorldParams;
use crate::game::state::{BuilderMode, SolutionState};
use crate::game::systems::gameplay::GameplayPlayGate;
use crate::game::ui::core::text_input::InlineTextEditState;
use crate::game::ui::{AreaKind, InventoryItem};
use crate::game::world::grid::{BlockSettings, TeleportSettings};
use crate::shared::config::{ActionKeyName, GameConfig};

pub use crate::game::local_player::{BlockSettingsClipboard, SelectionToolSwap};

/// 处理复制/粘贴配置与选区工具切换快捷键
pub fn clipboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    gate: GameplayPlayGate,
    inline_edit: Res<InlineTextEditState>,
    mut player: ClipboardPlayerMut,
    mut world: EditableWorldParams,
    mut solution_state: ResMut<SolutionState>,
) {
    if !gate.allows_world_edit() || inline_edit.is_active() {
        return;
    }

    if config
        .chord(ActionKeyName::ToggleSelectionTool)
        .just_triggered(&keys)
    {
        let slot = &mut player.inventory.hotbar[player.placement.selected];
        if slot.as_ref().and_then(|item| item.area()) == Some(AreaKind::Selection) {
            if let Some(previous) = player.tool_swap.displaced.take() {
                *slot = previous;
                player.placement.selection.clear();
            }
        } else {
            player.tool_swap.displaced = Some(*slot);
            *slot = Some(InventoryItem::Area(AreaKind::Selection));
        }
        return;
    }

    // 拖动选区时 Ctrl+C 由选区逻辑立即复制方块，不碰配置剪贴板
    if player.placement.selection.drag.is_some() {
        return;
    }

    if *player.builder_mode != BuilderMode::Edit
        && solution_state.entry != crate::game::state::WorldEntryMode::Free
    {
        return;
    }

    let Some(pos) = player.placement.target.map(|target| target.pos) else {
        return;
    };

    if config.chord(ActionKeyName::Copy).just_triggered(&keys) {
        let Some(block) = world.world.system_blocks().get(&pos) else {
            return;
        };
        let Some(settings) = world
            .world
            .block_settings()
            .get(&pos)
            .cloned()
            .or_else(|| block.kind.default_settings(pos))
        else {
            return;
        };
        player.clipboard.0 = Some(sanitize_clipboard_settings(settings));
        return;
    }

    if config.chord(ActionKeyName::Paste).just_triggered(&keys) {
        let Some(copied) = player.clipboard.0.clone() else {
            return;
        };
        let Some(block) = world.world.system_blocks().get(&pos).copied() else {
            return;
        };
        let Some(current) = world
            .world
            .block_settings()
            .get(&pos)
            .cloned()
            .or_else(|| block.kind.default_settings(pos))
        else {
            return;
        };
        if std::mem::discriminant(&copied) != std::mem::discriminant(&current) {
            return;
        }
        apply_block_settings_edit(player.edit_history.as_mut(), &mut world, pos, |blocks| {
            match copied {
                BlockSettings::Teleport(TeleportSettings { name, .. }) => {
                    blocks.set_teleport_pair(pos, None);
                    let mut settings = blocks.teleport_settings(pos);
                    settings.name = name;
                    blocks.set_teleport_settings(pos, settings);
                }
                other => blocks.set_block_settings(pos, other),
            }
        });
        solution_state.dirty = true;
    }
}

/// 剪贴时去掉传送门配对，避免粘出坏链接
fn sanitize_clipboard_settings(settings: BlockSettings) -> BlockSettings {
    match settings {
        BlockSettings::Teleport(TeleportSettings { name, .. }) => {
            BlockSettings::Teleport(TeleportSettings { name, pair: None })
        }
        other => other,
    }
}
