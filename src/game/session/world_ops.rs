use bevy::prelude::*;

use crate::game::simulation::markers::refresh_static_generated_markers;
use crate::game::state::{
    BuilderMode, GameMode, SimulationState, SolutionState, StartMenuScreen, WorldEntryMode,
};
use crate::game::ui::InventoryItems;
use crate::game::world::grid::WorldBlocks;
use crate::shared::save::{
    LoadedSave, PlayerSave, SaveKind, SaveSlot, SaveState, has_solutions_for_puzzle,
    invalidate_solutions_for_puzzle, next_named_save, puzzle_names, reset_solution_world,
    save_puzzle, save_solution, solution_names_for_puzzle,
};

use super::world_access::{PlayingWorldParams, SessionStateParams};

pub enum SaveCurrentWorldResult {
    Saved,
    NeedsPuzzleConfirm,
    Failed,
}

pub fn puzzle_save_needs_confirm(save_state: &SaveState) -> bool {
    save_state.current_kind == Some(SaveKind::Puzzle)
        && save_state
            .current
            .as_ref()
            .is_some_and(|slot| has_solutions_for_puzzle(&slot.puzzle))
}

pub fn save_current_world(
    world: &WorldBlocks,
    inventory: &InventoryItems,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &SimulationState,
    player: Option<PlayerSave>,
) -> SaveCurrentWorldResult {
    if puzzle_save_needs_confirm(save_state) {
        return SaveCurrentWorldResult::NeedsPuzzleConfirm;
    }
    if commit_save_current_world(
        world,
        inventory,
        save_state,
        solution_state,
        simulation,
        false,
        player,
    ) {
        SaveCurrentWorldResult::Saved
    } else {
        SaveCurrentWorldResult::Failed
    }
}

pub fn save_current_world_invalidate_solutions(
    world: &WorldBlocks,
    inventory: &InventoryItems,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &SimulationState,
    player: Option<PlayerSave>,
) -> bool {
    commit_save_current_world(
        world,
        inventory,
        save_state,
        solution_state,
        simulation,
        true,
        player,
    )
}

fn commit_save_current_world(
    world: &WorldBlocks,
    inventory: &InventoryItems,
    save_state: &mut SaveState,
    solution_state: &mut SolutionState,
    simulation: &SimulationState,
    invalidate_solutions: bool,
    player: Option<PlayerSave>,
) -> bool {
    let world = simulation.authoring_world(world);
    let kind = save_state.current_kind.unwrap_or(SaveKind::Puzzle);
    let mut slot = save_state.current.clone().unwrap_or_else(|| {
        SaveSlot::puzzle(next_named_save(&puzzle_names(&save_state.entries), "world"))
    });
    let saved = match kind {
        SaveKind::Puzzle => {
            if invalidate_solutions {
                invalidate_solutions_for_puzzle(&slot.puzzle);
            }
            save_puzzle(world, &slot, &inventory.to_saved_hotbar(), player)
        }
        SaveKind::Solution => {
            if slot.solution.is_none() {
                let Some(puzzle_id) = solution_state
                    .puzzle_id
                    .clone()
                    .or_else(|| Some(slot.puzzle.clone()))
                else {
                    return false;
                };
                slot = SaveSlot::solution(
                    &puzzle_id,
                    &next_named_save(
                        &solution_names_for_puzzle(&save_state.entries, &puzzle_id),
                        "solution",
                    ),
                );
            }
            save_solution(world, &slot, &inventory.to_saved_hotbar(), player)
        }
    };
    if saved {
        save_state.current = Some(slot);
        save_state.current_kind = Some(kind);
        solution_state.dirty = false;
        save_state.refresh();
    }
    saved
}

/// 切回编辑模式并重建场景
pub fn switch_to_edit_mode_and_rebuild(
    playing: &mut PlayingWorldParams,
    session: &mut SessionStateParams,
) {
    if let Some(puzzle_snapshot) = &session.solution_state.puzzle_snapshot {
        *playing.world = puzzle_snapshot.clone();
        refresh_static_generated_markers(&mut playing.world);
    }
    *session.builder_mode = BuilderMode::Edit;
    session.inventory.return_to_edit();
    session.carried.clear();
    session.placement.selected = 0;
    session.save_state.current_kind = Some(SaveKind::Puzzle);
    session.solution_state.puzzle_snapshot = None;
    session.solution_state.puzzle_id = None;
    session.playing_ui.paused = true;
    playing.clear_sim_sidecars();
    playing.rebuild_scene();
}

/// 重置当前解法世界到谜题快照
pub fn reset_current_solution(playing: &mut PlayingWorldParams, session: &mut SessionStateParams) {
    let Some(puzzle_snapshot) = &session.solution_state.puzzle_snapshot else {
        return;
    };
    reset_solution_world(&mut playing.world, puzzle_snapshot);
    refresh_static_generated_markers(&mut playing.world);
    session.simulation.running = false;
    session.simulation.step_requested = false;
    session.simulation.turn = 0;
    session.simulation.accumulator = 0.0;
    session.simulation.start_snapshot = None;
    session.simulation.start_structures = None;
    playing.clear_sim_sidecars();
    playing.rebuild_scene();
}

/// 清空已加载世界并回到主菜单
pub fn exit_to_main_menu(
    playing: &mut PlayingWorldParams,
    session: &mut SessionStateParams,
    next_state: &mut NextState<GameMode>,
    start_menu_screen: &mut StartMenuScreen,
) {
    clear_loaded_world(playing, session);
    *start_menu_screen = StartMenuScreen::Main;
    next_state.set(GameMode::StartMenu);
}

/// 把解码后的存档灌入当前会话
pub fn load_world_into_session(
    playing: &mut PlayingWorldParams,
    session: &mut SessionStateParams,
    slot: &SaveSlot,
    entry: WorldEntryMode,
    loaded: LoadedSave,
    current_mode: GameMode,
    next_state: &mut NextState<GameMode>,
) {
    let lighting = loaded.lighting;
    *playing.world = crate::game::world::grid::WorldBlocks(loaded.world);

    session.simulation.running = false;
    session.simulation.step_requested = false;
    session.simulation.turn = 0;
    session.simulation.accumulator = 0.0;
    session.simulation.start_snapshot = None;
    session.simulation.start_structures = None;
    session.placement.selection.clear();
    session.placement.edit_gesture = None;
    session.carried.clear();

    *session.builder_mode = match entry {
        WorldEntryMode::EditPuzzle => BuilderMode::Edit,
        WorldEntryMode::PlaySolution => BuilderMode::Play,
    };
    *session.inventory = InventoryItems::for_mode(*session.builder_mode);
    if let Some(hotbar) = loaded.hotbar {
        session.inventory.apply_saved_hotbar(hotbar);
    }
    session.placement.selected = 0;

    session.save_state.current = Some(slot.clone());
    session.save_state.current_kind = Some(match entry {
        WorldEntryMode::EditPuzzle => SaveKind::Puzzle,
        WorldEntryMode::PlaySolution => SaveKind::Solution,
    });
    session.save_state.select_puzzle(None);

    session.solution_state.entry = entry;
    session.solution_state.dirty = false;
    session.solution_state.puzzle_id = loaded.puzzle_id;
    session.solution_state.puzzle_snapshot = match entry {
        WorldEntryMode::EditPuzzle => None,
        WorldEntryMode::PlaySolution => loaded
            .puzzle_snapshot
            .map(crate::game::world::grid::WorldBlocks),
    };
    session.pending_player.0 = loaded.player;

    playing.commands.insert_resource(lighting);

    refresh_static_generated_markers(&mut playing.world);
    playing.clear_sim_sidecars();

    match current_mode {
        GameMode::StartMenu => next_state.set(GameMode::Playing),
        GameMode::Playing => playing.rebuild_scene(),
    }
}

/// 清空已加载世界与会话 sidecar（不切换 GameMode）
pub fn clear_loaded_world(playing: &mut PlayingWorldParams, session: &mut SessionStateParams) {
    session.simulation.running = false;
    session.simulation.step_requested = false;
    session.simulation.accumulator = 0.0;
    session.simulation.start_snapshot = None;
    session.simulation.start_structures = None;
    session.placement.selection.clear();
    session.placement.edit_gesture = None;
    playing.world.clear();
    session.save_state.current = None;
    session.save_state.current_kind = None;
    session.save_state.select_puzzle(None);
    playing
        .commands
        .insert_resource(crate::shared::save::PuzzleLighting::default());
    session.solution_state.puzzle_snapshot = None;
    session.solution_state.puzzle_id = None;
    session.solution_state.dirty = false;
    session.solution_state.entry = WorldEntryMode::EditPuzzle;
    playing.clear_sim_sidecars();
    playing.rebuild_scene();
}
