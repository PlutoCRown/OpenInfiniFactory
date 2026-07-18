//! 编辑模式配置复制粘贴，以及选区工具快捷切换

use bevy::prelude::*;

use crate::game::block_editing::world_refresh::apply_block_settings_edit;
use crate::game::edit_history::EditHistory;
use crate::game::session::PlayingWorldParams;
use crate::game::state::{
    BuilderMode, GameMode, PlacementState, PlayingUiState, SimulationState, SolutionState,
};
use crate::game::ui::core::text_input::InlineTextEditState;
use crate::game::ui::{AreaKind, InventoryItem, InventoryItems, UiRuntime};
use crate::game::world::grid::{BlockSettings, TeleportSettings};
use crate::shared::config::{ActionKeyName, GameConfig};

/// 系统方块配置剪贴板
#[derive(Resource, Default)]
pub struct BlockSettingsClipboard(pub Option<BlockSettings>);

/// Ctrl+X 临时换成选区工具时，记下被替换的快捷栏物品（含空槽）
#[derive(Resource, Default)]
pub struct SelectionToolSwap {
    pub displaced: Option<Option<InventoryItem>>,
}

/// 处理复制/粘贴配置与选区工具切换快捷键
pub fn clipboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    simulation: Res<SimulationState>,
    ui_runtime: Res<UiRuntime>,
    inline_edit: Res<InlineTextEditState>,
    builder_mode: Res<BuilderMode>,
    mut placement: ResMut<PlacementState>,
    mut inventory: ResMut<InventoryItems>,
    mut clipboard: ResMut<BlockSettingsClipboard>,
    mut tool_swap: ResMut<SelectionToolSwap>,
    mut world: PlayingWorldParams,
    mut edit_history: ResMut<EditHistory>,
    mut solution_state: ResMut<SolutionState>,
) {
    if *mode.get() != GameMode::Playing
        || !playing_ui.active_play()
        || simulation.is_active()
        || ui_runtime.blocks_gameplay()
        || inline_edit.is_active()
    {
        return;
    }

    if config
        .chord(ActionKeyName::ToggleSelectionTool)
        .just_triggered(&keys)
    {
        let slot = &mut inventory.hotbar[placement.selected];
        if slot.as_ref().and_then(|item| item.area()) == Some(AreaKind::Selection) {
            if let Some(previous) = tool_swap.displaced.take() {
                *slot = previous;
                placement.selection.clear();
            }
        } else {
            tool_swap.displaced = Some(*slot);
            *slot = Some(InventoryItem::Area(AreaKind::Selection));
        }
        return;
    }

    // 拖动选区时 Ctrl+C 由选区逻辑立即复制方块，不碰配置剪贴板
    if placement.selection.drag.is_some() {
        return;
    }

    if *builder_mode != BuilderMode::Edit {
        return;
    }

    let Some(pos) = placement.target.map(|target| target.pos) else {
        return;
    };

    if config.chord(ActionKeyName::Copy).just_triggered(&keys) {
        let Some(block) = world.world.system_blocks.get(&pos) else {
            return;
        };
        let Some(settings) = world
            .world
            .block_settings
            .get(&pos)
            .cloned()
            .or_else(|| block.kind.default_settings(pos))
        else {
            return;
        };
        clipboard.0 = Some(sanitize_clipboard_settings(settings));
        return;
    }

    if config.chord(ActionKeyName::Paste).just_triggered(&keys) {
        let Some(copied) = clipboard.0.clone() else {
            return;
        };
        let Some(block) = world.world.system_blocks.get(&pos).copied() else {
            return;
        };
        let Some(current) = world
            .world
            .block_settings
            .get(&pos)
            .cloned()
            .or_else(|| block.kind.default_settings(pos))
        else {
            return;
        };
        if std::mem::discriminant(&copied) != std::mem::discriminant(&current) {
            return;
        }
        apply_block_settings_edit(edit_history.as_mut(), &mut world, pos, |blocks| {
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
