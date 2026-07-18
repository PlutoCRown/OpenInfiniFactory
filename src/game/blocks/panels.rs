use bevy::prelude::*;

use crate::game::state::UiPanelId;

pub struct BlockPanelHooks {
    pub panel: UiPanelId,
    pub spawn_panel: fn(&mut ChildSpawnerCommands),
    pub spawn_overlays: fn(&mut ChildSpawnerCommands),
    pub register: fn(&mut App),
}

inventory::collect!(BlockPanelHooks);

/// 按面板 id 查找方块面板挂载钩子
pub fn find_block_panel_hooks(panel: UiPanelId) -> Option<&'static BlockPanelHooks> {
    inventory::iter::<BlockPanelHooks>.into_iter().find(|hooks| hooks.panel == panel)
}

pub fn register_all_panels(app: &mut App) {
    for hooks in inventory::iter::<BlockPanelHooks> {
        (hooks.register)(app);
    }
}
