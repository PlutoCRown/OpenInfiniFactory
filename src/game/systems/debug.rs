use crate::game::systems::perf::PerfStats;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::game::player::controller::{FlyCamera, player_collision_box};
use crate::game::simulation::stats::SimulationStepStats;
use crate::game::simulation::structure_state::StructureState;
use crate::game::state::{BuilderMode, GameMode, PlayingUiState, SimulationState};
use crate::game::ui::core::host::PlayingUiRootEntity;
use crate::game::ui::{PendingKeyBind, TextPromptState};
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::{
    BlockEntity, SceneChunkMeshes, WorldRenderAssets, despawn_world, rebuild_world_for_debug_state,
};
use crate::shared::config::{ActionKeyName, GameConfig};

const DEBUG_PANEL_WIDTH: f32 = 430.0;
const DEBUG_FONT_SIZE: f32 = 16.0;

#[derive(Resource, Default)]
pub struct DebugState {
    pub enabled: bool,
    pub factory_activity: bool,
}

#[derive(Resource, Clone)]
pub struct DebugFont(pub Handle<Font>);

#[derive(Component)]
pub struct DebugPanel;

#[derive(Component)]
pub struct DebugText;

pub fn load_debug_font(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    let font = Font::from_bytes(bevy::text::DEFAULT_FONT_DATA.to_vec());
    commands.insert_resource(DebugFont(fonts.add(font)));
}

pub fn setup_debug_ui(
    mut commands: Commands,
    debug_font: Res<DebugFont>,
    playing_ui_root: Res<PlayingUiRootEntity>,
) {
    // 挂在 PlayingUiRoot 下，由 PlayingUiCamera 渲染；独立根节点在菜单相机关闭后不会显示
    commands.entity(playing_ui_root.0).with_children(|root| {
        root.spawn((
            Text::new(""),
            TextFont {
                font: debug_font.0.clone().into(),
                font_size: FontSize::Px(DEBUG_FONT_SIZE),
                ..default()
            },
            TextColor(Color::srgb(0.95, 1.0, 0.72)),
            TextLayout::no_wrap(),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(18.0),
                top: Val::Px(14.0),
                width: Val::Px(DEBUG_PANEL_WIDTH),
                display: Display::None,
                ..default()
            },
            GlobalZIndex(25_000),
            Pickable::IGNORE,
            DebugPanel,
            DebugText,
        ));
    });
}

pub fn toggle_debug(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    pending_key_bind: Res<PendingKeyBind>,
    text_prompt: Res<TextPromptState>,
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    mut debug: ResMut<DebugState>,
) {
    if pending_key_bind.0.is_some()
        || text_prompt.is_open()
        || !in_playing(*mode.get(), &playing_ui)
    {
        return;
    }

    if keys.just_pressed(config.key(ActionKeyName::Debug).key_code()) {
        debug.enabled = !debug.enabled;
    }
}

pub fn toggle_factory_activity_debug(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    pending_key_bind: Res<PendingKeyBind>,
    text_prompt: Res<TextPromptState>,
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    mut debug: ResMut<DebugState>,
    mut structure_state: ResMut<StructureState>,
    simulation: Res<SimulationState>,
    mut commands: Commands,
    world: Res<WorldBlocks>,
    mut meshes: ResMut<Assets<Mesh>>,
    render_assets: Option<Res<WorldRenderAssets>>,
    block_entities: Query<Entity, With<BlockEntity>>,
    mut block_index: ResMut<crate::scene::BlockEntityIndex>,
    mut scene_chunks: ResMut<SceneChunkMeshes>,
) {
    if pending_key_bind.0.is_some()
        || text_prompt.is_open()
        || !in_playing(*mode.get(), &playing_ui)
    {
        return;
    }
    let Some(render_assets) = render_assets.as_ref() else {
        return;
    };

    if keys.just_pressed(config.key(ActionKeyName::DebugStructure).key_code()) {
        debug.factory_activity = !debug.factory_activity;
        if debug.factory_activity {
            if structure_state.is_empty() {
                structure_state.rebuild_factory_for_debug(&world);
            }
        } else if structure_state.is_empty() || !simulation.is_active() {
            structure_state.clear();
        }
        despawn_world(
            &mut commands,
            &mut meshes,
            &block_entities,
            &mut block_index,
            &mut scene_chunks,
        );
        rebuild_world_for_debug_state(
            &mut commands,
            &mut meshes,
            &world,
            &render_assets,
            &debug,
            &structure_state,
            &mut block_index,
            &mut scene_chunks,
        );
    }
}

fn in_playing(_mode: GameMode, _playing_ui: &PlayingUiState) -> bool {
    true
}

pub fn update_debug_ui(
    debug: Res<DebugState>,
    perf: ResMut<PerfStats>,
    diagnostics: Res<DiagnosticsStore>,
    world: Res<WorldBlocks>,
    builder_mode: Res<BuilderMode>,
    simulation: Res<SimulationState>,
    sim_stats: Res<SimulationStepStats>,
    player: Query<&Transform, With<FlyCamera>>,
    block_entities: Query<Entity, With<BlockEntity>>,
    mut panel: Query<(&mut Text, &mut Node), With<DebugPanel>>,
    mut refresh_at: Local<Option<std::time::Instant>>,
) {
    let Ok((mut text, mut style)) = panel.single_mut() else {
        return;
    };

    let next_display = if debug.enabled {
        Display::Flex
    } else {
        Display::None
    };
    if style.display != next_display {
        style.display = next_display;
    }

    if !debug.enabled {
        return;
    }

    // 调试字每帧改都会逼 UI 重新排版；限到约 10Hz
    let now = std::time::Instant::now();
    if refresh_at.is_some_and(|at| now.duration_since(at).as_millis() < 100) {
        return;
    }
    *refresh_at = Some(now);

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
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

    let render_remainder_ms = (perf.render_other_ms() - perf.render_gap_ms()).max(0.0);

    let next = format!(
        "Debug\nFPS: {:>4.0}\nFrame: {:>5.2} ms\nMain: {:>5.2} ms\n{}\n  Schedule/Untracked: {:>8.2} us\nRender/Engine: {:>5.2} ms\n  Frame Gap: {:>5.2} ms\n  Timing Remainder: {:>5.2} ms{}\nBlocks: {}  Entities: {}\nPlayer: {:.1}, {:.1}, {:.1}\n/: toggle",
        fps,
        perf.frame_ms(),
        perf.main_ms(),
        perf.format_scope_section(),
        perf.main_other_ms() * 1000.0,
        perf.render_other_ms(),
        perf.render_gap_ms(),
        render_remainder_ms,
        sim_turn_text,
        world.blocks.len(),
        block_entities.iter().count(),
        player_pos.x,
        player_pos.y,
        player_pos.z
    );
    if text.0 != next {
        text.0 = next;
    }
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
