use bevy::prelude::*;

use crate::game::block_editing::OpenBlockPanelDropdown;
use crate::game::block_editing::color_slot_ui::update_color_select_dropdowns;
use crate::game::block_editing::widgets::update_material_slot_hover;
use crate::game::blocks::panels::register_all_panels;
use crate::game::state::UiPanelId;
use crate::game::ui::access::ui;
use crate::game::ui::core::host::PlayingUiRootEntity;
use crate::game::ui::core::runtime::UiNavigation;
use crate::game::ui::core::text_input::InlineTextEditState;

/// 方块属性面板刷新阶段；各系统自行声明 UiContext。
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockPanelSystems;

/// 玩法交互发出的方块面板打开请求
#[derive(Message)]
pub struct OpenBlockPanelRequest {
    pub pos: IVec3,
    pub panel: UiPanelId,
}

/// 仅在打开非设置类方块面板时跑面板刷新
fn block_panel_systems_active(ui_navigation: Res<UiNavigation>) -> bool {
    ui_navigation
        .active_panel()
        .is_some_and(|panel| !panel.is_settings())
}

/// 消费延后打开请求并挂载方块面板
fn process_pending_block_panel_open(
    mut commands: Commands,
    mut requests: MessageReader<OpenBlockPanelRequest>,
    playing_ui_root: Option<Res<PlayingUiRootEntity>>,
) {
    let Some(request) = requests.read().last() else {
        return;
    };
    let root = playing_ui_root.as_ref().map(|root| root.0);
    ui.mount_block_panel(&mut commands, root, request.panel, request.pos);
}

pub struct BlockPanelsPlugin;

impl Plugin for BlockPanelsPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            BlockPanelSystems
                .after(process_pending_block_panel_open)
                .run_if(block_panel_systems_active),
        )
        .insert_resource(OpenBlockPanelDropdown::default())
        .insert_resource(InlineTextEditState::default())
        .add_message::<OpenBlockPanelRequest>()
        .add_systems(
            Update,
            process_pending_block_panel_open.in_set(crate::game::schedule::GameSet::Menus),
        )
        .add_systems(
            Update,
            (update_material_slot_hover, update_color_select_dropdowns).in_set(BlockPanelSystems),
        );
        register_all_panels(app);
    }
}
