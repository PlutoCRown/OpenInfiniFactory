use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::game::player::controller::{player_collision_box, FlyCamera};
use crate::game::simulation::factory_activity::FactoryStructureState;
use crate::game::simulation::runtime::{MovementPreview, SimulationStepStats};
use crate::game::state::{BuilderMode, GameMode, SimulationState};
use crate::game::ui::{PendingKeyBind, UiRuntime};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    despawn_world, rebuild_world_for_debug_state, BlockEntity, WorldRenderManager,
};
use crate::shared::config::{ConfigAction, GameConfig};

#[derive(Resource, Default)]
pub struct DebugState {
    pub enabled: bool,
    pub factory_activity: bool,
}

#[derive(Component)]
pub struct DebugPanel;

pub fn setup_debug_ui(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        },
        TextColor(Color::srgb(0.95, 1.0, 0.72)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(18.0),
            top: Val::Px(14.0),
            display: Display::None,
            ..default()
        },
        DebugPanel,
    ));
}

pub fn toggle_debug(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    pending_key_bind: Res<PendingKeyBind>,
    ui_runtime: Res<UiRuntime>,
    mode: Res<GameMode>,
    mut debug: ResMut<DebugState>,
) {
    if pending_key_bind.0.is_some() || ui_runtime.text_prompt().is_some() || !gameplay_mode(*mode) {
        return;
    }

    if keys.just_pressed(config.key(ConfigAction::Debug).key_code()) {
        debug.enabled = !debug.enabled;
    }
}

pub fn toggle_factory_activity_debug(
    keys: Res<ButtonInput<KeyCode>>,
    pending_key_bind: Res<PendingKeyBind>,
    ui_runtime: Res<UiRuntime>,
    mode: Res<GameMode>,
    mut debug: ResMut<DebugState>,
    mut factory_structures: ResMut<FactoryStructureState>,
    mut commands: Commands,
    world: Res<WorldBlocks>,
    mut meshes: ResMut<Assets<Mesh>>,
    render_manager: Res<WorldRenderManager>,
    block_entities: Query<Entity, With<BlockEntity>>,
) {
    if pending_key_bind.0.is_some() || ui_runtime.text_prompt().is_some() || !gameplay_mode(*mode) {
        return;
    }

    if keys.just_pressed(KeyCode::KeyP) {
        debug.factory_activity = !debug.factory_activity;
        factory_structures.ensure_current_world(&world);
        despawn_world(&mut commands, &block_entities);
        rebuild_world_for_debug_state(
            &mut commands,
            &mut meshes,
            &world,
            &render_manager,
            &debug,
            &factory_structures,
        );
    }
}

fn gameplay_mode(mode: GameMode) -> bool {
    matches!(
        mode,
        GameMode::Playing | GameMode::Inventory | GameMode::Paused
    )
}

pub fn update_debug_ui(
    debug: Res<DebugState>,
    diagnostics: Res<DiagnosticsStore>,
    world: Res<WorldBlocks>,
    builder_mode: Res<BuilderMode>,
    simulation: Res<SimulationState>,
    sim_stats: Res<SimulationStepStats>,
    movement_preview: Res<MovementPreview>,
    player: Query<&Transform, With<FlyCamera>>,
    block_entities: Query<Entity, With<BlockEntity>>,
    mut panel: Query<(&mut Text, &mut Node), With<DebugPanel>>,
) {
    let Ok((mut text, mut style)) = panel.single_mut() else {
        return;
    };

    style.display = if debug.enabled {
        Display::Flex
    } else {
        Display::None
    };
    style.top = if debug.factory_activity {
        Val::Auto
    } else {
        Val::Px(14.0)
    };
    style.bottom = if debug.factory_activity {
        Val::Px(18.0)
    } else {
        Val::Auto
    };

    if !debug.enabled {
        return;
    }

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
        .unwrap_or(0.0);
    let frame_ms = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|frame_time| frame_time.smoothed())
        .unwrap_or(0.0);

    let player_pos = player
        .single()
        .map(|transform| transform.translation)
        .unwrap_or(Vec3::ZERO);

    let sim_turn_text = if *builder_mode == BuilderMode::Play
        && simulation.running
        && sim_stats.has_sample
    {
        format!(
            "\n\nSim Turn (last)\n  Total: {:>5.2} ms\n  Prep: {:>5.2} ms\n  Gravity: {:>5.2} ms\n  Signals: {:>5.2} ms\n  Markers A: {:>5.2} ms\n  Mark Move: {:>5.2} ms\n  Exec Move: {:>5.2} ms\n  Markers B: {:>5.2} ms\n  Behavior: {:>5.2} ms\n  Signals End: {:>5.2} ms\n  Rebuild: {:>5.2} ms",
            sim_stats.total_ms,
            sim_stats.prep_ms,
            sim_stats.gravity_ms,
            sim_stats.signal_ms,
            sim_stats.marker_before_move_ms,
            sim_stats.movement_mark_ms,
            sim_stats.movement_execute_ms,
            sim_stats.marker_after_move_ms,
            sim_stats.behavior_ms,
            sim_stats.signal_refresh_ms,
            sim_stats.render_rebuild_ms,
        )
    } else {
        String::new()
    };

    let moments_text = if debug.factory_activity && !movement_preview.moments.is_empty() {
        let rows = movement_preview
            .moments
            .iter()
            .map(|moment| {
                let source = moment
                    .source
                    .map(|pos| format!(" @ {},{},{}", pos.x, pos.y, pos.z))
                    .unwrap_or_default();
                format!("  {}{}", moment.label, source)
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("\n\nMoments\n{rows}")
    } else {
        String::new()
    };

    text.0 = format!(
        "Debug\nFPS: {:>4.0}\nFrame: {:>5.2} ms{}\nBlocks: {}  Entities: {}\nPlayer: {:.1}, {:.1}, {:.1}{}\n/: toggle",
        fps,
        frame_ms,
        sim_turn_text,
        world.blocks.len(),
        block_entities.iter().count(),
        player_pos.x,
        player_pos.y,
        player_pos.z,
        moments_text
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

    let Ok(transform) = player.single() else {
        return;
    };

    let (min, max) = player_collision_box(transform.translation);
    let center = (min + max) * 0.5;
    let size = max - min;
    gizmos.cube(
        Transform::from_translation(center).with_scale(size),
        Color::srgb(1.0, 0.1, 0.1),
    );
}
