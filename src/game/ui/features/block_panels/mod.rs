use bevy::prelude::*;

use crate::game::block_editing::OpenBlockPanelDropdown;
use crate::game::block_editing::color_slot_ui::update_color_select_dropdowns;
use crate::game::block_editing::widgets::update_material_slot_hover;
use crate::game::blocks::panels::register_all_panels;
use crate::game::state::UiPanelId;
use crate::game::systems::perf::PerfScope;
use crate::game::ui::access::{UiAccessScope, UiMainThread, ui};
use crate::game::ui::core::host::PlayingUiRootEntity;
use crate::game::ui::core::runtime::UiRuntime;
use crate::game::ui::core::text_input::InlineTextEditState;

/// 方块属性面板刷新（须在 UiAccessScope 内）
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockPanelSystems;

/// 放置输入请求打开的方块面板（延后到 UiAccessScope 再挂载，以便 i18n）
#[derive(Resource, Default)]
pub struct PendingBlockPanelOpen(pub Option<(IVec3, UiPanelId)>);

/// 仅在打开非设置类方块面板时跑面板刷新
fn block_panel_systems_active(ui_runtime: Res<UiRuntime>) -> bool {
    ui_runtime
        .active_panel()
        .is_some_and(|panel| !panel.is_settings())
}

/// 消费延后打开请求并挂载方块面板
fn process_pending_block_panel_open(
    _ui_thread: UiMainThread,
    mut pending: ResMut<PendingBlockPanelOpen>,
    playing_ui_root: Option<Res<PlayingUiRootEntity>>,
) {
    let Some((pos, panel)) = pending.0.take() else {
        return;
    };
    let root = playing_ui_root.as_ref().map(|root| root.0);
    ui.mount_block_panel(root, panel, pos);
}

pub struct BlockPanelsPlugin;

impl Plugin for BlockPanelsPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            BlockPanelSystems
                .in_set(UiAccessScope)
                .after(process_pending_block_panel_open)
                .run_if(block_panel_systems_active),
        )
        .insert_resource(OpenBlockPanelDropdown::default())
        .insert_resource(InlineTextEditState::default())
        .insert_resource(PendingBlockPanelOpen::default())
        .add_systems(
            Update,
            process_pending_block_panel_open
                .in_set(UiAccessScope)
                .after(PerfScope::Placement)
                .before(PerfScope::Menus),
        )
        .add_systems(
            Update,
            (
                update_material_slot_hover,
                update_color_select_dropdowns,
            )
                .in_set(BlockPanelSystems),
        );
        register_all_panels(app);
    }
}
