use bevy::prelude::*;

use crate::game::state::UiPanelId;

pub struct BlockPanelHooks {
    pub panel: UiPanelId,
    pub spawn_panel: fn(&mut ChildSpawnerCommands),
    pub spawn_overlays: fn(&mut ChildSpawnerCommands),
    pub register: fn(&mut App),
}

inventory::collect!(BlockPanelHooks);

pub fn spawn_all_panels(root: &mut ChildSpawnerCommands) {
    let mut panels = inventory::iter::<BlockPanelHooks>.into_iter().collect::<Vec<_>>();
    panels.sort_by_key(|hooks| hooks.panel as u8);
    for hooks in panels {
        (hooks.spawn_panel)(root);
    }
}

pub fn spawn_all_overlays(root: &mut ChildSpawnerCommands) {
    for hooks in inventory::iter::<BlockPanelHooks> {
        (hooks.spawn_overlays)(root);
    }
}

pub fn register_all_panels(app: &mut App) {
    for hooks in inventory::iter::<BlockPanelHooks> {
        (hooks.register)(app);
    }
}
