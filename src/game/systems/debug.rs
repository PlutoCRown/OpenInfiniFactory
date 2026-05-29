use std::time::{Duration, Instant};

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::game::player::controller::{player_collision_box, FlyCamera};
use crate::game::simulation::runtime::SimulationStepStats;
use crate::game::state::{BuilderMode, GameMode, SimulationState};
use crate::game::ui::PendingKeyBind;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::rendering::BlockEntity;
use crate::shared::config::{ConfigAction, GameConfig};

#[derive(Resource, Default)]
pub struct DebugState {
    pub enabled: bool,
}

#[derive(Component)]
pub struct DebugPanel;

#[derive(Resource)]
pub struct PerfStats {
    frame_started: Instant,
    mark: Instant,
    frame_ms: SmoothedMs,
    main_ms: SmoothedMs,
    input_ms: SmoothedMs,
    menu_ms: SmoothedMs,
    simulation_ms: SmoothedMs,
    view_ms: SmoothedMs,
    animation_ms: SmoothedMs,
    ui_ms: SmoothedMs,
    debug_ms: SmoothedMs,
    main_other_ms: SmoothedMs,
    render_other_ms: SmoothedMs,
    display_timer: Timer,
    display_text: String,
}

impl Default for PerfStats {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            frame_started: now,
            mark: now,
            frame_ms: SmoothedMs::default(),
            main_ms: SmoothedMs::default(),
            input_ms: SmoothedMs::default(),
            menu_ms: SmoothedMs::default(),
            simulation_ms: SmoothedMs::default(),
            view_ms: SmoothedMs::default(),
            animation_ms: SmoothedMs::default(),
            ui_ms: SmoothedMs::default(),
            debug_ms: SmoothedMs::default(),
            main_other_ms: SmoothedMs::default(),
            render_other_ms: SmoothedMs::default(),
            display_timer: Timer::from_seconds(0.25, TimerMode::Repeating),
            display_text: String::new(),
        }
    }
}

#[derive(Default)]
struct SmoothedMs {
    value: f64,
    initialized: bool,
}

impl SmoothedMs {
    fn sample(&mut self, duration: Duration) {
        let ms = duration.as_secs_f64() * 1000.0;
        self.sample_ms(ms);
    }

    fn sample_ms(&mut self, ms: f64) {
        if self.initialized {
            self.value = self.value * 0.86 + ms * 0.14;
        } else {
            self.value = ms;
            self.initialized = true;
        }
    }
}

pub fn begin_perf_frame(mut perf: ResMut<PerfStats>) {
    let now = Instant::now();
    let frame_duration = now.saturating_duration_since(perf.frame_started);
    perf.frame_ms.sample(frame_duration);
    perf.frame_started = now;
    perf.mark = now;
}

pub fn mark_perf_input(mut perf: ResMut<PerfStats>) {
    let elapsed = perf.mark_elapsed();
    perf.input_ms.sample(elapsed);
}

pub fn mark_perf_menus(mut perf: ResMut<PerfStats>) {
    let elapsed = perf.mark_elapsed();
    perf.menu_ms.sample(elapsed);
}

pub fn mark_perf_simulation(mut perf: ResMut<PerfStats>) {
    let elapsed = perf.mark_elapsed();
    perf.simulation_ms.sample(elapsed);
}

pub fn mark_perf_view(mut perf: ResMut<PerfStats>) {
    let elapsed = perf.mark_elapsed();
    perf.view_ms.sample(elapsed);
}

pub fn mark_perf_animation(mut perf: ResMut<PerfStats>) {
    let elapsed = perf.mark_elapsed();
    perf.animation_ms.sample(elapsed);
}

pub fn mark_perf_ui(mut perf: ResMut<PerfStats>) {
    let elapsed = perf.mark_elapsed();
    perf.ui_ms.sample(elapsed);
}

pub fn mark_perf_debug(mut perf: ResMut<PerfStats>) {
    let elapsed = perf.mark_elapsed();
    perf.debug_ms.sample(elapsed);
}

pub fn finish_perf_frame(mut perf: ResMut<PerfStats>) {
    let main_ms = Instant::now()
        .saturating_duration_since(perf.frame_started)
        .as_secs_f64()
        * 1000.0;
    let render_other_ms = (perf.frame_ms.value - main_ms).max(0.0);
    let measured_main_ms = perf.input_ms.value
        + perf.menu_ms.value
        + perf.simulation_ms.value
        + perf.view_ms.value
        + perf.animation_ms.value
        + perf.ui_ms.value
        + perf.debug_ms.value;
    let main_other_ms = (main_ms - measured_main_ms).max(0.0);
    perf.main_ms.sample_ms(main_ms);
    perf.main_other_ms.sample_ms(main_other_ms);
    perf.render_other_ms.sample_ms(render_other_ms);
}

impl PerfStats {
    fn mark_elapsed(&mut self) -> Duration {
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.mark);
        self.mark = now;
        elapsed
    }
}

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
    time: Res<Time>,
    debug: Res<DebugState>,
    mut perf: ResMut<PerfStats>,
    diagnostics: Res<DiagnosticsStore>,
    world: Res<WorldBlocks>,
    builder_mode: Res<BuilderMode>,
    simulation: Res<SimulationState>,
    sim_stats: Res<SimulationStepStats>,
    player: Query<&Transform, With<FlyCamera>>,
    block_entities: Query<Entity, With<BlockEntity>>,
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

    perf.display_timer.tick(time.delta());
    if !perf.display_timer.finished() && !perf.display_text.is_empty() {
        text.sections[0].value.clone_from(&perf.display_text);
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

    perf.display_text = format!(
        "Debug\nFPS: {:>4.0}\nFrame: {:>5.2} ms\nMain: {:>5.2} ms\n  Input: {:>5.2} ms\n  Menus: {:>5.2} ms\n  Sim Systems: {:>5.2} ms\n  View: {:>5.2} ms\n  Anim: {:>5.2} ms\n  UI: {:>5.2} ms\n  Debug UI: {:>5.2} ms\n  Schedule/Other: {:>5.2} ms\nRender/Engine: {:>5.2} ms{}\nBlocks: {}  Entities: {}\nPlayer: {:.1}, {:.1}, {:.1}\n/: toggle",
        fps,
        perf.frame_ms.value,
        perf.main_ms.value,
        perf.input_ms.value,
        perf.menu_ms.value,
        perf.simulation_ms.value,
        perf.view_ms.value,
        perf.animation_ms.value,
        perf.ui_ms.value,
        perf.debug_ms.value,
        perf.main_other_ms.value,
        perf.render_other_ms.value,
        sim_turn_text,
        world.blocks.len(),
        block_entities.iter().count(),
        player_pos.x,
        player_pos.y,
        player_pos.z
    );
    text.sections[0].value.clone_from(&perf.display_text);
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
