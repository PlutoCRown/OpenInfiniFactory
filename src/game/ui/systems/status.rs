use bevy::ecs::system::SystemParam;

use crate::game::simulation::movement::PusherState;
use crate::game::simulation::movement_plan::target_structure_movement_lines;
use crate::game::simulation::runtime::SignalNetworkCache;
use crate::game::simulation::structure_state::StructureState;
use crate::game::simulation::structures::MovementInfluenceCache;
use crate::game::systems::debug::DebugState;
use crate::game::world::factory_registry::FactoryBlockRegistry;

fn builder_mode_name(mode: BuilderMode) -> String {
    i18n.t(match mode {
        BuilderMode::Edit => "mode.edit",
        BuilderMode::Play => "mode.play",
    })
}

#[derive(SystemParam)]
pub(crate) struct MovementPreviewDeps<'w> {
    debug: Res<'w, DebugState>,
    simulation: Res<'w, SimulationState>,
    structure_state: Res<'w, StructureState>,
    factory_registry: Res<'w, FactoryBlockRegistry>,
    signal_cache: ResMut<'w, SignalNetworkCache>,
    pusher_state: Res<'w, PusherState>,
    movement_influence: Res<'w, MovementInfluenceCache>,
}

pub fn update_status_ui(
    _ui_thread: UiMainThread,
    placement: Res<PlacementState>,
    world: Res<WorldBlocks>,
    builder_mode: Res<BuilderMode>,
    save_state: Res<SaveState>,
    mut movement_preview: MovementPreviewDeps,
    mut texts: Query<(&StatusText, &mut Text)>,
) {
    for (status, mut text) in &mut texts {
        text.0 = match status.0 {
            StatusTextKind::TargetMovement => target_movement_status_text(
                *builder_mode,
                &placement,
                &world,
                &mut movement_preview,
            ),
            kind => status_text_value(kind, &placement, &world, *builder_mode, &save_state),
        };
    }
}

fn status_text_value(
    kind: StatusTextKind,
    placement: &PlacementState,
    world: &WorldBlocks,
    builder_mode: BuilderMode,
    save_state: &SaveState,
) -> String {
    match kind {
        StatusTextKind::Summary => {
            let world_name = save_state
                .current
                .as_ref()
                .map(|name| name.clone())
                .unwrap_or_else(|| i18n.t("save.no_world_loaded"));
            i18n.fmt(
                "status.world_mode",
                &[
                    ("world", world_name),
                    ("mode", builder_mode_name(builder_mode)),
                ],
            )
        }
        StatusTextKind::TargetBlock => target_status_line(placement, world),
        StatusTextKind::TargetMovement => String::new(),
    }
}

fn target_movement_status_text(
    builder_mode: BuilderMode,
    placement: &PlacementState,
    world: &WorldBlocks,
    deps: &mut MovementPreviewDeps,
) -> String {
    if builder_mode != BuilderMode::Play
        || !deps.debug.factory_activity
        || !deps.simulation.is_active()
        || deps.simulation.running
    {
        return String::new();
    }
    let Some(hit) = placement.target.as_ref() else {
        return String::new();
    };
    let solution = deps.simulation.start_snapshot.as_ref().unwrap_or(world);
    let solution_structures = deps
        .simulation
        .start_structures
        .as_ref()
        .unwrap_or(&*deps.structure_state);
    let Some(lines) = target_structure_movement_lines(
        hit.pos,
        world,
        &*deps.structure_state,
        solution,
        solution_structures,
        &*deps.factory_registry,
        &mut deps.signal_cache,
        &*deps.pusher_state,
        &*deps.movement_influence,
    ) else {
        return String::new();
    };
    let mut text = i18n.t("status.target_movement.title");
    text.push('\n');
    if lines.is_empty() {
        text.push_str(&i18n.t("status.target_movement.none"));
    } else {
        text.push_str(&lines.join("\n"));
    }
    text
}
