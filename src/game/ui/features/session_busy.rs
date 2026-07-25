//! 会话忙碌遮罩：加载中 / 保存中

use bevy::prelude::*;

use crate::game::session::{SessionBusy, SessionBusyCover};
use crate::game::systems::perf::PerfScope;
use crate::game::ui::access::{UiAccessScope, UiMainThread, i18n};
use crate::game::ui::components::text;

/// 全屏忙碌遮罩根节点
#[derive(Component)]
pub struct SessionBusyOverlay;

/// 全屏封面底图
#[derive(Component)]
pub struct SessionBusyCoverImage;

/// 半透明灰幕（叠在封面上）
#[derive(Component)]
pub struct SessionBusyDim;

/// 居中提示文本
#[derive(Component)]
pub struct SessionBusyLabel;

/// 在菜单 / 游玩 UI 根下各挂一份遮罩（相机切换后仍能看见）
pub fn spawn_session_busy_overlay(root: &mut ChildSpawnerCommands, busy: SessionBusy) {
    let display = if busy.is_busy() {
        Display::Flex
    } else {
        Display::None
    };
    let label = busy.label_key().map(|key| i18n.t(key)).unwrap_or_default();
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            display,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
        GlobalZIndex(100_000),
        SessionBusyOverlay,
        Pickable::default(),
    ))
    .with_children(|overlay| {
        overlay.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::None,
                ..default()
            },
            ImageNode {
                image_mode: NodeImageMode::Stretch,
                ..default()
            },
            SessionBusyCoverImage,
            Pickable::IGNORE,
        ));
        overlay
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.07, 0.1, 0.55)),
                SessionBusyDim,
                Pickable::IGNORE,
            ))
            .with_children(|dim| {
                dim.spawn((
                    text(label, 22.0, Color::WHITE),
                    TextLayout::no_wrap(),
                    SessionBusyLabel,
                ));
            });
    });
}

/// 按 SessionBusy 切换遮罩显隐、文案与封面
pub fn update_session_busy_overlay(
    _ui_thread: UiMainThread,
    busy: Res<SessionBusy>,
    cover: Res<SessionBusyCover>,
    mut overlays: Query<&mut Node, (With<SessionBusyOverlay>, Without<SessionBusyCoverImage>)>,
    mut cover_images: Query<
        (&mut ImageNode, &mut Node),
        (With<SessionBusyCoverImage>, Without<SessionBusyOverlay>),
    >,
    mut labels: Query<&mut Text, With<SessionBusyLabel>>,
) {
    let busy_changed = busy.is_changed();
    let cover_changed = cover.is_changed();

    let display = if busy.is_busy() {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in &mut overlays {
        if node.display != display {
            node.display = display;
        }
    }

    // 新挂载的 Playing 遮罩也要立刻贴上已有封面
    if busy_changed || cover_changed || busy.is_busy() {
        let cover_handle = cover.image.as_ref().filter(|_| busy.is_busy());
        for (mut image_node, mut node) in &mut cover_images {
            if let Some(handle) = cover_handle {
                if image_node.image != *handle {
                    *image_node = ImageNode {
                        image: handle.clone(),
                        image_mode: NodeImageMode::Stretch,
                        ..default()
                    };
                }
                if node.display != Display::Flex {
                    node.display = Display::Flex;
                }
            } else if node.display != Display::None {
                *image_node = ImageNode::default();
                node.display = Display::None;
            }
        }
    }

    if !busy_changed {
        return;
    }
    let Some(key) = busy.label_key() else {
        return;
    };
    let label = i18n.t(key);
    for mut text in &mut labels {
        if text.as_str() != label {
            *text = Text::new(label.clone());
        }
    }
}

pub struct SessionBusyUiPlugin;

impl Plugin for SessionBusyUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_session_busy_overlay
                .in_set(UiAccessScope)
                .after(PerfScope::Animation)
                .before(PerfScope::Ui),
        );
    }
}
