use bevy::prelude::*;

use super::layout::transparent_node;

/// UI 通用图标句柄（白色图标 + 透明底，可按需染色）
#[derive(Resource, Clone)]
pub struct UiIconAssets {
    pub crosshair: Handle<Image>,
    pub edit: Handle<Image>,
    pub delete: Handle<Image>,
    pub close: Handle<Image>,
}

/// 挂载关闭叉图标（image 须在 UiAccessScope 内先取好）
pub fn spawn_close_icon(parent: &mut ChildSpawnerCommands, image: Handle<Image>) {
    spawn_ui_icon(parent, image, 12.0);
}

/// 挂载固定尺寸的 UI 图标图
pub fn spawn_ui_icon(parent: &mut ChildSpawnerCommands, image: Handle<Image>, size: f32) {
    parent.spawn((
        transparent_node(Node {
            width: Val::Px(size),
            height: Val::Px(size),
            flex_shrink: 0.0,
            ..default()
        }),
        ImageNode::new(image),
        Pickable::IGNORE,
    ));
}
