use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::game::player::controller::{player_collision_box, FlyCamera};
use crate::game::state::GameMode;
use crate::game::ui::PendingKeyBind;
use crate::shared::config::{ConfigAction, GameConfig};

#[derive(Resource, Default)]
pub struct DebugState {
    pub enabled: bool,
}

#[derive(Component)]
pub struct DebugPanel;

pub fn setup_debug_ui(mut commands: Commands) {
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font_size: 16.0,
                    color: Color::srgb(0.95, 1.0, 0.72),
                    ..default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(18.0),
                top: Val::Px(14.0),
                display: Display::None,
                ..default()
            },
            ..default()
        },
        DebugPanel,
    ));
}

pub fn toggle_debug(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mode: Res<GameMode>,
    pending_key_bind: Res<PendingKeyBind>,
    mut debug: ResMut<DebugState>,
) {
    if *mode == GameMode::Settings && pending_key_bind.0.is_some() {
        return;
    }

    if keys.just_pressed(config.key(ConfigAction::Debug).key_code()) {
        debug.enabled = !debug.enabled;
    }
}

pub fn update_debug_ui(
    debug: Res<DebugState>,
    diagnostics: Res<DiagnosticsStore>,
    player: Query<&Transform, With<FlyCamera>>,
    mut panel: Query<(&mut Text, &mut Style), With<DebugPanel>>,
) {
    let Ok((mut text, mut style)) = panel.get_single_mut() else {
        return;
    };

    style.display = if debug.enabled {
        Display::Flex
    } else {
        Display::None
    };

    if !debug.enabled {
        return;
    }

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
        .unwrap_or(0.0);

    let player_pos = player
        .get_single()
        .map(|transform| transform.translation)
        .unwrap_or(Vec3::ZERO);

    text.sections[0].value = format!(
        "Debug\nFPS: {:>5.1}\nPlayer: {:.2}, {:.2}, {:.2}\n/: toggle",
        fps, player_pos.x, player_pos.y, player_pos.z
    );
}

pub fn draw_player_collider(
    debug: Res<DebugState>,
    player: Query<&Transform, With<FlyCamera>>,
    mut gizmos: Gizmos,
) {
    if !debug.enabled {
        return;
    }

    let Ok(transform) = player.get_single() else {
        return;
    };

    let (min, max) = player_collision_box(transform.translation);
    let center = (min + max) * 0.5;
    let size = max - min;
    gizmos.cuboid(
        Transform::from_translation(center).with_scale(size),
        Color::srgb(1.0, 0.1, 0.1),
    );
}
