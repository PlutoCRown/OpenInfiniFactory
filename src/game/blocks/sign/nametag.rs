//! 瞄准告示牌时的 MC 名牌式悬浮文本

use bevy::prelude::*;
use bevy::text::LineBreak;
use bevy::window::PrimaryWindow;

use crate::game::player::controller::FlyCamera;
use crate::game::state::{GameMode, PlayingUiState};
use crate::game::systems::gameplay::AimFocus;
use crate::game::ui::UiRuntime;
use crate::game::ui::components::default_font_size;
use crate::game::world::grid::grid_to_world;

/// 悬浮名牌根节点（绝对定位到准星瞄准的告示格中心）
#[derive(Component)]
pub struct SignNametagRoot;

/// 名牌文本
#[derive(Component)]
pub struct SignNametagText;

/// 在游玩 HUD 下生成名牌（默认隐藏）
pub fn spawn_sign_nametag(root: &mut ChildSpawnerCommands) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            max_width: Val::Percent(50.0),
            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            display: Display::None,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        // left/top 钉在格中心，再按自身宽高回退 50% 实现居中
        UiTransform::from_translation(Val2::percent(-50.0, -50.0)),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
        SignNametagRoot,
        GlobalZIndex(2),
        Pickable::IGNORE,
    ))
    .with_children(|tag| {
        tag.spawn((
            Text::new(""),
            Node {
                width: Val::Percent(100.0),
                ..default()
            },
            TextFont {
                font_size: default_font_size(16.0),
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::new(Justify::Center, LineBreak::WordOrCharacter),
            SignNametagText,
            Pickable::IGNORE,
        ));
    });
}

/// 瞄准带文本的告示牌时，把名牌钉在该格中心的屏幕投影上
pub fn sync_sign_nametag(
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    ui_runtime: Res<UiRuntime>,
    aim: Res<AimFocus>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<FlyCamera>>,
    mut root: Query<&mut Node, With<SignNametagRoot>>,
    mut text: Query<&mut Text, With<SignNametagText>>,
) {
    let Ok(mut style) = root.single_mut() else {
        return;
    };

    let hide = *mode.get() != GameMode::Playing
        || !playing_ui.active_play()
        || ui_runtime.blocks_gameplay();
    let label = aim.sign_label.as_deref().filter(|_| !hide);
    let pos = aim.hit.map(|hit| hit.pos).filter(|_| label.is_some());

    let (Some(label), Some(pos)) = (label, pos) else {
        if style.display != Display::None {
            style.display = Display::None;
        }
        return;
    };

    let Ok((camera, cam_tf)) = camera.single() else {
        style.display = Display::None;
        return;
    };
    let Ok(window) = windows.single() else {
        style.display = Display::None;
        return;
    };
    // 3D 渲到 physical 尺寸的离屏图；UI 用 logical。需把 viewport 坐标映到窗口逻辑像素。
    let Some(viewport_size) = camera.logical_viewport_size() else {
        style.display = Display::None;
        return;
    };
    if viewport_size.x <= 0.0 || viewport_size.y <= 0.0 {
        style.display = Display::None;
        return;
    }
    let world_pos = grid_to_world(pos);
    let Ok(screen) = camera.world_to_viewport(cam_tf, world_pos) else {
        style.display = Display::None;
        return;
    };
    let ui_size = Vec2::new(window.width(), window.height());
    let ui_pos = screen / viewport_size * ui_size;

    for mut text in &mut text {
        if text.0 != label {
            text.0 = label.to_string();
        }
    }

    // 不超过半屏宽，超长由 TextLayout 换行
    style.max_width = Val::Px((window.width() * 0.5).max(80.0));
    style.left = Val::Px(ui_pos.x);
    style.top = Val::Px(ui_pos.y);
    if style.display != Display::Flex {
        style.display = Display::Flex;
    }
}
