/// 启动时加载 UI 通用图标
pub fn load_ui_icons(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(crate::game::ui::components::UiIconAssets {
        crosshair: asset_server.load("ui/icons/crosshair.png"),
        edit: asset_server.load("ui/icons/edit.png"),
        delete: asset_server.load("ui/icons/delete.png"),
        close: asset_server.load("ui/icons/close.png"),
    });
}
