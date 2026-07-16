//! 会话忙碌遮罩：加载中 / 保存中

use bevy::prelude::*;

use crate::game::session::SessionBusy;
use crate::game::systems::perf::PerfScope;
use crate::game::ui::access::{i18n, I18nRevision, UiAccessScope, UiMainThread};
use crate::game::ui::components::text;

/// 全屏忙碌遮罩根节点
#[derive(Component)]
pub struct SessionBusyOverlay;

/// 居中提示文本
#[derive(Component)]
pub struct SessionBusyLabel;

/// 在菜单 / 游玩 UI 根下各挂一份遮罩（相机切换后仍能看见）
pub fn spawn_session_busy_overlay(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            display: Display::None,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.07, 0.1, 0.55)),
        GlobalZIndex(100_000),
        SessionBusyOverlay,
        Pickable::default(),
    ))
    .with_children(|overlay| {
        overlay.spawn((
            text("", 22.0, Color::WHITE),
            TextLayout::no_wrap(),
            SessionBusyLabel,
        ));
    });
}

/// 按 SessionBusy 切换遮罩显隐与文案
pub fn update_session_busy_overlay(
    _ui_thread: UiMainThread,
    busy: Res<SessionBusy>,
    i18n_rev: Res<I18nRevision>,
    mut overlays: Query<&mut Node, With<SessionBusyOverlay>>,
    mut labels: Query<&mut Text, With<SessionBusyLabel>>,
    added: Query<(), Added<SessionBusyOverlay>>,
) {
    if !(busy.is_changed() || i18n_rev.is_changed() || !added.is_empty()) {
        return;
    }

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
