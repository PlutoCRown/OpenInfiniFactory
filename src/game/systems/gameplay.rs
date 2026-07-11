use bevy::ecs::system::SystemParam;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::game::blocks::{BlockData, BlockKind};
use crate::game::player::controller::{
    player_intersects_block, teleport_player_preserve_offset, FlyCamera,
};
use crate::game::simulation::markers::refresh_static_generated_markers;
use crate::game::simulation::structure_state::{
    query_factory_structure, StructureFreedom, StructureKind, StructureState,
};
use crate::game::simulation::structures::material_structure;
use crate::game::state::{
    BuilderMode, EditGesture, EditGestureKind, GameMode, GameSettings, PlacementState,
    PlayingUiState, SelectionAxis, SelectionBounds, SelectionDrag, SimulationState, SolutionState,
};
use crate::game::systems::debug::DebugState;
use crate::game::ui::dismiss_playing_overlay;
use crate::game::ui::UiHost;
use crate::game::ui::{
    AreaKind, CarriedItem, InlineTextEditState, InventoryItems, OpenBlockPanelDropdown,
    OpenSettingsDropdown, PanelDragState, PendingKeyBind, TextPromptState, UiRuntime, HOTBAR_SLOTS,
};
use crate::game::world::animation::BlockAnimation;
use crate::game::world::grid::{
    grid_to_world, raycast_blocks, raycast_edit_drag_grid, MaterialWeld, TargetHit, WorldBlocks,
};
use crate::game::world::rendering::StructureBounds;
use crate::game::world::rendering::{
    block_face_highlight_transform, despawn_edit_previews, rebuild_world_for_debug_state,
    rebuild_world_with_animations, rebuild_world_with_animations_for_debug_state,
    spawn_block_preview, spawn_block_with_animation, spawn_delete_bounds_preview,
    spawn_edit_preview, AimFaceHighlight, BlockEntity, BlockEntityLayer, EditPreview, EditPreviewKind, HoverMarker,
    HoverStructureBounds, WorldRenderAssets,
};
use crate::scene::{refresh_edit_changes, BlockEntityIndex};
use crate::shared::config::{ConfigSelectionMode, GameConfig};

#[derive(SystemParam)]
pub struct PanelCloseDeps<'w> {
    ui_runtime: ResMut<'w, UiRuntime>,
    ui_host: ResMut<'w, UiHost>,
    open_block_dropdown: ResMut<'w, OpenBlockPanelDropdown>,
    open_settings_dropdown: ResMut<'w, OpenSettingsDropdown>,
    pending_key_bind: ResMut<'w, PendingKeyBind>,
    inline_edit: ResMut<'w, InlineTextEditState>,
    drag: ResMut<'w, PanelDragState>,
}

impl PanelCloseDeps<'_> {
    fn dismiss_overlay(
        &mut self,
        playing_ui: &mut PlayingUiState,
        carried: &mut CarriedItem,
        commands: &mut Commands,
    ) -> bool {
        dismiss_playing_overlay(
            playing_ui,
            carried,
            &mut self.ui_runtime,
            &mut self.ui_host,
            &mut self.open_block_dropdown,
            &mut self.open_settings_dropdown,
            &mut self.pending_key_bind,
            &mut self.inline_edit,
            &mut self.drag,
            commands,
        )
    }
}

#[derive(SystemParam)]
pub struct PlacementQueries<'w, 's> {
    meshes: ResMut<'w, Assets<Mesh>>,
    block_entities: Query<'w, 's, (Entity, &'static BlockEntity)>,
    edit_previews: Query<'w, 's, Entity, With<EditPreview>>,
    player: Query<'w, 's, (&'static mut FlyCamera, &'static mut Transform), With<FlyCamera>>,
    render_assets: Option<Res<'w, WorldRenderAssets>>,
    debug: Res<'w, DebugState>,
    structure_state: ResMut<'w, StructureState>,
    block_index: ResMut<'w, BlockEntityIndex>,
}

pub fn gameplay_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    config: Res<GameConfig>,
    text_prompt: Res<TextPromptState>,
    mode: Res<State<GameMode>>,
    mut playing_ui: ResMut<PlayingUiState>,
    mut placement: ResMut<PlacementState>,
    mut carried: ResMut<CarriedItem>,
    mut panel_close: PanelCloseDeps,
    mut simulation: ResMut<SimulationState>,
    mut commands: Commands,
) {
    let bindings = &config.key_bindings;

    let typing = panel_close.pending_key_bind.0.is_some()
        || text_prompt.is_open()
        || panel_close.inline_edit.is_active();
    if typing {
        mouse_wheel.clear();
        return;
    }

    if *mode.get() != GameMode::Playing {
        mouse_wheel.clear();
        return;
    }

    if keys.just_pressed(bindings.pause.key_code()) {
        if panel_close.dismiss_overlay(&mut playing_ui, &mut carried, &mut commands) {
            // Overlay dismissed.
        } else {
            playing_ui.paused = !playing_ui.paused;
            if playing_ui.paused {
                simulation.running = false;
                simulation.step_requested = false;
                simulation.speed = 1.0;
            }
        }
    }

    if keys.just_pressed(bindings.inventory.key_code()) {
        if panel_close.dismiss_overlay(&mut playing_ui, &mut carried, &mut commands) {
            // Overlay dismissed.
        } else {
            playing_ui.inventory_open = true;
        }
    }

    if panel_close.ui_runtime.blocks_gameplay() || !playing_ui.active_play() {
        mouse_wheel.clear();
        return;
    }

    for (key, index) in [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
        (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6),
        (KeyCode::Digit8, 7),
        (KeyCode::Digit9, 8),
    ] {
        if keys.just_pressed(key) && index < HOTBAR_SLOTS {
            if placement.selected != index {
                placement.selection.clear();
                placement.edit_gesture = None;
                placement.selected = index;
            }
        }
    }

    let wheel_delta: f32 = mouse_wheel.read().map(|event| event.y).sum();
    if wheel_delta.abs() > f32::EPSILON {
        let direction = if wheel_delta > 0.0 { -1 } else { 1 };
        let selected = (placement.selected as i32 + direction).rem_euclid(HOTBAR_SLOTS as i32);
        if placement.selected != selected as usize {
            placement.selection.clear();
            placement.edit_gesture = None;
            placement.selected = selected as usize;
        }
    }
}

pub fn placement_input(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut world: ResMut<WorldBlocks>,
    mut solution_state: ResMut<SolutionState>,
    mut inventory: ResMut<InventoryItems>,
    config: Res<GameConfig>,
    builder_mode: Res<BuilderMode>,
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    simulation: Res<SimulationState>,
    mut placement: ResMut<PlacementState>,
    mut ui_runtime: ResMut<UiRuntime>,
    queries: PlacementQueries,
) {
    let PlacementQueries {
        mut meshes,
        block_entities,
        edit_previews,
        mut player,
        render_assets,
        debug,
        mut structure_state,
        mut block_index,
    } = queries;

    if *mode.get() != GameMode::Playing || !playing_ui.active_play() {
        placement.edit_gesture = None;
        despawn_edit_previews(&mut commands, &edit_previews);
        return;
    }
    let Some(render_assets) = render_assets.as_ref() else {
        return;
    };
    let place_button = config
        .input(crate::shared::config::ActionKeyName::Place)
        .mouse_button()
        .unwrap_or(MouseButton::Left);
    let delete_button = config
        .input(crate::shared::config::ActionKeyName::Delete)
        .mouse_button()
        .unwrap_or(MouseButton::Right);
    let pick_button = config
        .input(crate::shared::config::ActionKeyName::Pick)
        .mouse_button()
        .unwrap_or(MouseButton::Middle);

    if ui_runtime.blocks_gameplay() || simulation.is_active() {
        placement.edit_gesture = None;
        despawn_edit_previews(&mut commands, &edit_previews);
        return;
    }

    if selected_area(&inventory, &placement) != Some(AreaKind::Selection) {
        placement.selection.clear();
    }

    let edit_selection_mode = placement
        .edit_gesture
        .as_ref()
        .map(|gesture| match gesture.kind {
            EditGestureKind::Place { .. } => config.place_selection_mode,
            EditGestureKind::Delete => config.delete_selection_mode,
        });

    let current_place_at = placement.target.map(|target| {
        if edit_selection_mode.is_some_and(|mode| mode != ConfigSelectionMode::Point) {
            target.pos
        } else {
            target.pos + target.normal
        }
    });
    let current_delete_at = placement.target.map(|target| target.pos);
    let current_target_pos = placement.target.map(|target| target.pos);
    let force_place = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if mouse_buttons.just_pressed(place_button)
        && !force_place
        && open_target_block_ui(current_target_pos, &world, *builder_mode, &mut ui_runtime)
    {
        placement.edit_gesture = None;
        placement.selection.clear();
        despawn_edit_previews(&mut commands, &edit_previews);
        return;
    }

    if selected_area(&inventory, &placement) == Some(AreaKind::Selection) {
        if handle_selection_area_input(
            &mouse_buttons,
            current_target_pos,
            place_button,
            &mut placement,
            &mut world,
            &block_entities,
            &mut commands,
            &mut meshes,
            &render_assets,
            &debug,
            &mut structure_state,
            &mut block_index,
        ) {
            solution_state.dirty = true;
        }
        despawn_edit_previews(&mut commands, &edit_previews);
        spawn_selection_previews(&placement, &mut commands, &render_assets);
        return;
    }

    placement.selection.clear();

    if mouse_buttons.just_pressed(pick_button) {
        if let Some(pos) = current_target_pos {
            pick_target_block(pos, &world, &mut placement, &mut inventory);
        }
        placement.edit_gesture = None;
        despawn_edit_previews(&mut commands, &edit_previews);
        return;
    }

    if *builder_mode == BuilderMode::Play
        && !simulation.is_active()
        && keys.just_pressed(config.key_bindings.alternate.key_code())
    {
        if let Some(pos) = current_target_pos {
            if alternate_block_at(
                pos,
                &mut world,
                &block_entities,
                &mut commands,
                &mut meshes,
                &render_assets,
                &debug,
                &mut structure_state,
                &mut block_index,
            ) {
                solution_state.dirty = true;
            }
        }
        placement.edit_gesture = None;
        despawn_edit_previews(&mut commands, &edit_previews);
        return;
    }

    if keys.just_pressed(config.key_bindings.rotate_or_rollback.key_code()) {
        let reverse_rotation = shift_pressed(&keys);
        if let Some(gesture) = placement.edit_gesture.as_mut() {
            if let EditGestureKind::Place { block } = &mut gesture.kind {
                if block.kind.is_directional() {
                    block.facing = rotate_facing(block.facing, reverse_rotation);
                    placement.preview_facing = block.facing;
                }
            }
        } else if let Some(pos) = current_target_pos {
            if rotate_block_at(
                pos,
                reverse_rotation,
                &mut world,
                &block_entities,
                &mut commands,
                &mut meshes,
                &render_assets,
                &debug,
                &mut structure_state,
                &mut block_index,
            ) {
                solution_state.dirty = true;
            } else if selected_place_block(&inventory, *builder_mode, &placement)
                .is_some_and(|block| block.kind.is_directional())
            {
                placement.preview_facing =
                    rotate_facing(placement.preview_facing, reverse_rotation);
            }
        } else if selected_place_block(&inventory, *builder_mode, &placement)
            .is_some_and(|block| block.kind.is_directional())
        {
            placement.preview_facing = rotate_facing(placement.preview_facing, reverse_rotation);
        }
    }

    if mouse_buttons.just_pressed(delete_button)
        && *builder_mode == BuilderMode::Play
        && try_player_teleport(current_target_pos, &world, &mut player)
    {
        placement.edit_gesture = None;
        despawn_edit_previews(&mut commands, &edit_previews);
        return;
    }

    if mouse_buttons.just_pressed(delete_button) {
        match placement.edit_gesture.as_mut() {
            Some(gesture) if matches!(gesture.kind, EditGestureKind::Place { .. }) => {
                gesture.canceled = true;
            }
            None => {
                if let Some(start) = current_delete_at {
                    let plane_normal = placement
                        .target
                        .and_then(|target| (target.normal != IVec3::ZERO).then_some(target.normal))
                        .unwrap_or(IVec3::Y);
                    placement.edit_gesture = Some(EditGesture {
                        kind: EditGestureKind::Delete,
                        start,
                        canceled: false,
                        plane_normal,
                    });
                }
            }
            Some(_) => {}
        }
    }

    if mouse_buttons.just_pressed(place_button) {
        match placement.edit_gesture.as_mut() {
            Some(gesture) if matches!(gesture.kind, EditGestureKind::Delete) => {
                gesture.canceled = true;
            }
            None => {
                if let Some(start) = current_place_at {
                    if let Some(block) = selected_place_block(&inventory, *builder_mode, &placement)
                    {
                        let plane_normal = placement
                            .target
                            .and_then(|target| {
                                (target.normal != IVec3::ZERO).then_some(target.normal)
                            })
                            .unwrap_or(IVec3::Y);
                        placement.edit_gesture = Some(EditGesture {
                            kind: EditGestureKind::Place { block },
                            start,
                            canceled: false,
                            plane_normal,
                        });
                    }
                }
            }
            Some(_) => {}
        }
    }

    let released_place = mouse_buttons.just_released(place_button);
    let released_delete = mouse_buttons.just_released(delete_button);
    let should_finish = placement.edit_gesture.as_ref().is_some_and(|gesture| {
        matches!(gesture.kind, EditGestureKind::Place { .. }) && released_place
            || matches!(gesture.kind, EditGestureKind::Delete) && released_delete
    });

    let player_pos = player
        .single()
        .ok()
        .map(|(_, transform)| transform.translation);

    if should_finish {
        if let Some(gesture) = placement.edit_gesture.take() {
            if !gesture.canceled {
                if commit_edit_gesture(
                    gesture,
                    current_place_at,
                    current_delete_at,
                    &config,
                    &mut world,
                    *builder_mode,
                    player_pos,
                    &mut commands,
                    &mut meshes,
                    &render_assets,
                    &debug,
                    &mut structure_state,
                    &mut block_index,
                ) {
                    solution_state.dirty = true;
                }
            }
        }
    }

    despawn_edit_previews(&mut commands, &edit_previews);
    if let Some(gesture) = &placement.edit_gesture {
        if !gesture.canceled {
            spawn_gesture_previews(
                gesture,
                current_place_at,
                current_delete_at,
                &config,
                &world,
                *builder_mode,
                player_pos,
                &mut commands,
                &mut meshes,
                &render_assets,
            );
        }
    }
}

fn selected_place_block(
    inventory: &InventoryItems,
    builder_mode: BuilderMode,
    placement: &PlacementState,
) -> Option<BlockData> {
    let kind = inventory.hotbar[placement.selected]?;
    let kind = kind.block()?;
    can_place_in_mode(kind, builder_mode).then_some(BlockData::new(kind, placement.preview_facing))
}

fn selected_area(inventory: &InventoryItems, placement: &PlacementState) -> Option<AreaKind> {
    inventory.hotbar[placement.selected].and_then(|item| item.area())
}

fn try_player_teleport(
    target: Option<IVec3>,
    world: &WorldBlocks,
    player: &mut Query<(&mut FlyCamera, &mut Transform), With<FlyCamera>>,
) -> bool {
    let Some(pos) = target else {
        return false;
    };
    let Some(block) = world.system_blocks.get(&pos) else {
        return false;
    };
    if !matches!(
        block.kind,
        BlockKind::TeleportEntrance | BlockKind::TeleportExit
    ) {
        return false;
    }
    let Some(partner) = world.teleport_partner(pos) else {
        return false;
    };
    let Ok((mut camera, mut transform)) = player.single_mut() else {
        return false;
    };
    teleport_player_preserve_offset(pos, partner, &mut transform, &mut camera);
    true
}

fn open_target_block_ui(
    target: Option<IVec3>,
    world: &WorldBlocks,
    builder_mode: BuilderMode,
    ui_runtime: &mut UiRuntime,
) -> bool {
    if builder_mode != BuilderMode::Edit {
        return false;
    }

    let Some(pos) = target else {
        return false;
    };
    let Some(block) = world.system_blocks.get(&pos) else {
        return false;
    };

    let Some(panel) = block.kind.ui_panel() else {
        return false;
    };

    ui_runtime.open_block(panel, pos);
    true
}

fn pick_target_block(
    pos: IVec3,
    world: &WorldBlocks,
    placement: &mut PlacementState,
    inventory: &mut InventoryItems,
) {
    let Some(block) = world
        .blocks
        .get(&pos)
        .or_else(|| world.system_blocks.get(&pos))
    else {
        return;
    };
    let kind = block.kind;
    if !inventory.can_take_block(kind) {
        return;
    }

    if let Some(index) = inventory.hotbar_index_of_block(kind) {
        placement.selected = index;
    } else {
        inventory.set_hotbar_block(placement.selected, kind);
    }
    placement.selection.clear();
    placement.edit_gesture = None;
}

fn handle_selection_area_input(
    mouse_buttons: &ButtonInput<MouseButton>,
    current_target_pos: Option<IVec3>,
    place_button: MouseButton,
    placement: &mut PlacementState,
    world: &mut WorldBlocks,
    block_entities: &Query<(Entity, &BlockEntity)>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &mut StructureState,
    block_index: &mut BlockEntityIndex,
) -> bool {
    let mut changed = false;
    if let Some(drag) = placement.selection.drag.as_mut() {
        if let Some(current) = current_target_pos {
            if let Some((axis, offset)) = selection_drag_offset(*drag, current) {
                drag.axis = Some(axis);
                drag.offset = offset;
            }
        }
    }

    if mouse_buttons.just_released(place_button) {
        if let Some(drag) = placement.selection.drag.take() {
            if drag.offset != IVec3::ZERO {
                if let Some(bounds) = placement.selection.bounds {
                    if move_selection(
                        world,
                        block_entities,
                        commands,
                        meshes,
                        render_assets,
                        debug,
                        structure_state,
                        block_index,
                        bounds,
                        drag.offset,
                    ) {
                        placement.selection.bounds = Some(bounds.moved(drag.offset));
                        changed = true;
                    }
                }
            }
        }
        return changed;
    }

    if !mouse_buttons.just_pressed(place_button) {
        return false;
    }

    let Some(pos) = current_target_pos else {
        return false;
    };

    if let Some(bounds) = placement.selection.bounds {
        if bounds.contains(pos) {
            placement.selection.drag = Some(SelectionDrag {
                start: pos,
                axis: None,
                offset: IVec3::ZERO,
            });
            return false;
        }
    }

    if let Some(first) = placement.selection.first_corner.take() {
        placement.selection.bounds = Some(SelectionBounds::from_corners(first, pos));
        placement.selection.drag = None;
    } else {
        placement.selection.first_corner = Some(pos);
        placement.selection.bounds = None;
        placement.selection.drag = None;
    }
    false
}

fn selection_drag_offset(drag: SelectionDrag, current: IVec3) -> Option<(SelectionAxis, IVec3)> {
    let delta = current - drag.start;
    if delta == IVec3::ZERO {
        return None;
    }
    let axis = drag.axis.unwrap_or_else(|| strongest_axis(delta));
    let offset = match axis {
        SelectionAxis::X => axis.offset(delta.x),
        SelectionAxis::Y => axis.offset(delta.y),
        SelectionAxis::Z => axis.offset(delta.z),
    };
    Some((axis, offset))
}

fn strongest_axis(delta: IVec3) -> SelectionAxis {
    if delta.x.abs() >= delta.y.abs() && delta.x.abs() >= delta.z.abs() {
        SelectionAxis::X
    } else if delta.y.abs() >= delta.z.abs() {
        SelectionAxis::Y
    } else {
        SelectionAxis::Z
    }
}

fn move_selection(
    world: &mut WorldBlocks,
    block_entities: &Query<(Entity, &BlockEntity)>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &mut StructureState,
    block_index: &mut BlockEntityIndex,
    bounds: SelectionBounds,
    offset: IVec3,
) -> bool {
    let positions = bounds.positions();
    let selected: Vec<(IVec3, BlockData)> = positions
        .iter()
        .filter_map(|pos| world.blocks.get(pos).copied().map(|block| (*pos, block)))
        .collect();
    if selected.is_empty() {
        return true;
    }

    let selected_positions: std::collections::HashSet<IVec3> =
        selected.iter().map(|(pos, _)| *pos).collect();
    let updated_welds = moved_selection_welds(world, &selected_positions, offset);
    if selected.iter().any(|(pos, block)| {
        let target = *pos + offset;
        target.y < 0
            || (!selected_positions.contains(&target)
                && world
                    .blocks
                    .get(&target)
                    .is_some_and(|target_block| target_block.kind.has_collision()))
            || !block.kind.has_collision()
    }) {
        return false;
    }

    for (pos, _) in &selected {
        world.remove(pos);
        if let Some((entity, block_entity)) = block_entities
            .iter()
            .find(|(_, block_entity)| block_entity.pos == *pos)
        {
            match block_entity.layer {
                BlockEntityLayer::Animatable => {
                    block_index.remove_animatable(*pos);
                }
                BlockEntityLayer::System => {
                    block_index.remove_system(*pos);
                }
                BlockEntityLayer::Scene => {
                    block_index.remove_scene(*pos);
                }
            }
            commands.entity(entity).despawn();
        }
    }

    let mut animations = std::collections::HashMap::new();
    for (pos, block) in selected {
        let target = pos + offset;
        world.insert(target, block);
        let stored = world.blocks[&target];
        animations.insert(
            target,
            BlockAnimation {
                block_id: stored.id,
                from_pos: pos,
                to_pos: target,
                from_facing: block.facing,
                to_facing: block.facing,
                kind: crate::game::world::animation::BlockAnimationKind::Move,
                duration: None,
                progress: None,
            },
        );
    }
    world.replace_material_welds(updated_welds);
    if debug.factory_activity {
        despawn_block_entities(commands, block_entities, block_index);
        rebuild_world_with_animations_for_debug_state(
            commands,
            meshes,
            world,
            render_assets,
            &animations,
            debug,
            structure_state,
            block_index,
        );
    } else {
        for (target, animation) in animations {
            let block = world.blocks[&target];
            spawn_block_with_animation(
                commands,
                meshes,
                render_assets,
                world,
                target,
                block,
                Some(animation),
                None,
                block_index,
            );
        }
    }
    true
}

fn despawn_block_entities(
    commands: &mut Commands,
    block_entities: &Query<(Entity, &BlockEntity)>,
    block_index: &mut BlockEntityIndex,
) {
    block_index.clear();
    for (entity, _) in block_entities {
        commands.entity(entity).despawn();
    }
}

fn refresh_edit_generated_markers(world: &mut WorldBlocks) {
    refresh_static_generated_markers(world);
}

fn alternate_block_at(
    pos: IVec3,
    world: &mut WorldBlocks,
    block_entities: &Query<(Entity, &BlockEntity)>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &mut StructureState,
    block_index: &mut BlockEntityIndex,
) -> bool {
    let Some(block) = world.blocks.get_mut(&pos) else {
        return false;
    };
    let Some(kind) = block.kind.alternate() else {
        return false;
    };

    if matches!(
        (block.kind, kind),
        (BlockKind::Conveyor, BlockKind::ReverseConveyor)
            | (BlockKind::ReverseConveyor, BlockKind::Conveyor)
    ) {
        block.facing = block.facing.rotate().rotate();
    }
    block.kind = kind;
    refresh_edit_generated_markers(world);
    despawn_block_entities(commands, block_entities, block_index);
    rebuild_world_for_debug_state(
        commands,
        meshes,
        world,
        render_assets,
        debug,
        structure_state,
        block_index,
    );
    true
}

fn rotate_block_at(
    pos: IVec3,
    reverse: bool,
    world: &mut WorldBlocks,
    block_entities: &Query<(Entity, &BlockEntity)>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &mut StructureState,
    block_index: &mut BlockEntityIndex,
) -> bool {
    let Some(block) = world.blocks.get_mut(&pos) else {
        return false;
    };
    if !block.kind.is_directional() {
        return false;
    }

    let from_facing = block.facing;
    block.facing = rotate_facing(block.facing, reverse);
    let updated = *block;

    refresh_edit_generated_markers(world);
    let mut animations = std::collections::HashMap::new();
    animations.insert(
        pos,
        BlockAnimation {
            block_id: updated.id,
            from_pos: pos,
            to_pos: pos,
            from_facing,
            to_facing: updated.facing,
            kind: crate::game::world::animation::BlockAnimationKind::Move,
            duration: None,
            progress: None,
        },
    );

    despawn_block_entities(commands, block_entities, block_index);
    if debug.factory_activity {
        rebuild_world_with_animations_for_debug_state(
            commands,
            meshes,
            world,
            render_assets,
            &animations,
            debug,
            structure_state,
            block_index,
        );
    } else {
        rebuild_world_with_animations(
            commands,
            meshes,
            world,
            render_assets,
            &animations,
            None,
            block_index,
        );
    }
    true
}

fn rotate_facing(
    facing: crate::game::blocks::Facing,
    reverse: bool,
) -> crate::game::blocks::Facing {
    if reverse {
        facing.rotate_counter()
    } else {
        facing.rotate()
    }
}

fn shift_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)
}

fn moved_selection_welds(
    world: &WorldBlocks,
    selected_positions: &std::collections::HashSet<IVec3>,
    offset: IVec3,
) -> std::collections::HashSet<MaterialWeld> {
    world
        .material_welds
        .iter()
        .filter_map(|weld| {
            let move_a = selected_positions.contains(&weld.a);
            let move_b = selected_positions.contains(&weld.b);
            if move_a != move_b {
                return None;
            }
            let a = if move_a { weld.a + offset } else { weld.a };
            let b = if move_b { weld.b + offset } else { weld.b };
            (a != b).then_some(MaterialWeld::new(a, b))
        })
        .collect()
}

fn spawn_selection_previews(
    placement: &PlacementState,
    commands: &mut Commands,
    render_assets: &WorldRenderAssets,
) {
    if let Some(first) = placement.selection.first_corner {
        spawn_edit_preview(commands, render_assets, first, EditPreviewKind::Selection);
    }

    if let Some(bounds) = placement.selection.bounds {
        let offset = placement
            .selection
            .drag
            .map(|drag| drag.offset)
            .unwrap_or(IVec3::ZERO);
        for pos in bounds.moved(offset).positions() {
            spawn_edit_preview(commands, render_assets, pos, EditPreviewKind::Selection);
        }
    }
}

fn can_place_block_at(
    place_at: IVec3,
    block: BlockData,
    builder_mode: BuilderMode,
    world: &WorldBlocks,
    player_pos: Option<Vec3>,
) -> bool {
    if place_at.y < 0 {
        return false;
    }

    if !world.can_place_block_kind_at(place_at, block.kind) {
        return false;
    }

    if !can_place_in_mode(block.kind, builder_mode) {
        return false;
    }

    if let Some(position) = player_pos {
        if player_intersects_block(position, place_at) {
            return false;
        }
    }

    true
}

fn commit_edit_gesture(
    gesture: EditGesture,
    current_place_at: Option<IVec3>,
    current_delete_at: Option<IVec3>,
    config: &GameConfig,
    world: &mut WorldBlocks,
    builder_mode: BuilderMode,
    player_pos: Option<Vec3>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &mut StructureState,
    block_index: &mut BlockEntityIndex,
) -> bool {
    let mut changed_positions = std::collections::HashSet::new();
    match gesture.kind {
        EditGestureKind::Place { block } => {
            let positions = selection_positions(
                config.place_selection_mode,
                gesture.start,
                current_place_at.unwrap_or(gesture.start),
            );
            for pos in &positions {
                if can_place_block_at(*pos, block, builder_mode, world, player_pos) {
                    world.insert(*pos, block);
                    changed_positions.insert(*pos);
                }
            }
        }
        EditGestureKind::Delete => {
            let positions = selection_positions(
                config.delete_selection_mode,
                gesture.start,
                current_delete_at.unwrap_or(gesture.start),
            );
            for pos in &positions {
                if delete_block_at(*pos, builder_mode, world) {
                    changed_positions.insert(*pos);
                }
            }
        }
    }
    if changed_positions.is_empty() {
        return false;
    }
    refresh_edit_generated_markers(world);
    refresh_edit_changes(
        commands,
        meshes,
        block_index,
        world,
        render_assets,
        debug,
        structure_state,
        &changed_positions,
    );
    true
}

fn spawn_gesture_previews(
    gesture: &EditGesture,
    current_place_at: Option<IVec3>,
    current_delete_at: Option<IVec3>,
    config: &GameConfig,
    world: &WorldBlocks,
    builder_mode: BuilderMode,
    player_pos: Option<Vec3>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    render_assets: &WorldRenderAssets,
) {
    match gesture.kind {
        EditGestureKind::Place { block } => {
            let positions = selection_positions(
                config.place_selection_mode,
                gesture.start,
                current_place_at.unwrap_or(gesture.start),
            );
            let positions: Vec<IVec3> = positions
                .into_iter()
                .filter(|pos| can_place_block_at(*pos, block, builder_mode, world, player_pos))
                .collect();
            let preview_world = preview_world(world, &positions, block);
            for pos in positions {
                spawn_block_preview(commands, meshes, render_assets, &preview_world, pos, block);
            }
        }
        EditGestureKind::Delete => {
            let positions = selection_positions(
                config.delete_selection_mode,
                gesture.start,
                current_delete_at.unwrap_or(gesture.start),
            );
            let deletable: Vec<IVec3> = positions
                .into_iter()
                .filter(|pos| can_delete_at(*pos, builder_mode, world))
                .collect();
            if let Some(bounds) = SelectionBounds::from_positions(&deletable) {
                spawn_delete_bounds_preview(commands, render_assets, bounds.min, bounds.max);
            }
        }
    }
}

fn preview_world(world: &WorldBlocks, positions: &[IVec3], block: BlockData) -> WorldBlocks {
    let mut preview = world.clone();
    for pos in positions {
        preview.insert(*pos, block);
    }
    refresh_edit_generated_markers(&mut preview);
    preview
}

fn selection_positions(mode: ConfigSelectionMode, start: IVec3, end: IVec3) -> Vec<IVec3> {
    match mode {
        ConfigSelectionMode::Point => vec![start],
        ConfigSelectionMode::Line => line_selection(start, end),
        ConfigSelectionMode::Plane => plane_selection(start, end),
    }
}

fn line_selection(start: IVec3, end: IVec3) -> Vec<IVec3> {
    let delta = end - start;
    let axis = if delta.x.abs() >= delta.y.abs() && delta.x.abs() >= delta.z.abs() {
        0
    } else if delta.y.abs() >= delta.z.abs() {
        1
    } else {
        2
    };

    let (min, max) = match axis {
        0 => min_max(start.x, end.x),
        1 => min_max(start.y, end.y),
        _ => min_max(start.z, end.z),
    };

    (min..=max)
        .map(|value| match axis {
            0 => IVec3::new(value, start.y, start.z),
            1 => IVec3::new(start.x, value, start.z),
            _ => IVec3::new(start.x, start.y, value),
        })
        .collect()
}

fn plane_selection(start: IVec3, end: IVec3) -> Vec<IVec3> {
    let (min_x, max_x) = min_max(start.x, end.x);
    let (min_z, max_z) = min_max(start.z, end.z);
    let mut positions = Vec::new();
    for x in min_x..=max_x {
        for z in min_z..=max_z {
            positions.push(IVec3::new(x, start.y, z));
        }
    }
    positions
}

fn min_max(a: i32, b: i32) -> (i32, i32) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

pub fn apply_fov(
    settings: Res<GameSettings>,
    mut cameras: Query<&mut Projection, With<FlyCamera>>,
) {
    if !settings.is_changed() {
        return;
    }

    for mut projection in &mut cameras {
        if let Projection::Perspective(perspective) = projection.as_mut() {
            perspective.fov = settings.fov_degrees.to_radians();
        }
    }
}

#[derive(SystemParam)]
pub struct HoverPreviewDeps<'w, 's> {
    commands: Commands<'w, 's>,
    meshes: ResMut<'w, Assets<Mesh>>,
    render_assets: Option<Res<'w, WorldRenderAssets>>,
    inventory: Res<'w, InventoryItems>,
    builder_mode: Res<'w, BuilderMode>,
    player: Query<'w, 's, &'static Transform, With<FlyCamera>>,
    edit_previews: Query<'w, 's, Entity, With<EditPreview>>,
}

pub fn update_hover(
    mut placement: ResMut<PlacementState>,
    config: Res<GameConfig>,
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    ui_runtime: Res<UiRuntime>,
    debug: Res<DebugState>,
    camera: Query<
        &Transform,
        (
            With<FlyCamera>,
            Without<HoverMarker>,
            Without<AimFaceHighlight>,
        ),
    >,
    world: Res<WorldBlocks>,
    structure_state: Res<StructureState>,
    mut hover_bounds: ResMut<HoverStructureBounds>,
    mut marker: Query<
        (
            &mut Transform,
            &mut Visibility,
            &MeshMaterial3d<StandardMaterial>,
        ),
        (
            With<HoverMarker>,
            Without<FlyCamera>,
            Without<AimFaceHighlight>,
        ),
    >,
    mut aim_face: Query<
        (&mut Transform, &mut Visibility),
        (
            With<AimFaceHighlight>,
            Without<FlyCamera>,
            Without<HoverMarker>,
        ),
    >,
    mut preview_deps: HoverPreviewDeps,
) {
    if *mode.get() != GameMode::Playing || !playing_ui.active_play() || ui_runtime.blocks_gameplay()
    {
        placement.target = None;
        hover_bounds.bounds = None;
        if let Ok((_, mut visibility, _)) = marker.single_mut() {
            *visibility = Visibility::Hidden;
        }
        if let Ok((_, mut visibility)) = aim_face.single_mut() {
            *visibility = Visibility::Hidden;
        }
        despawn_edit_previews(&mut preview_deps.commands, &preview_deps.edit_previews);
        return;
    }

    let Ok(camera_transform) = camera.single() else {
        return;
    };

    let origin = camera_transform.translation;
    let dir = *camera_transform.forward();

    if let Some(gesture) = placement.edit_gesture.as_mut() {
        if !gesture.canceled {
            let selection_mode = match gesture.kind {
                EditGestureKind::Place { .. } => config.place_selection_mode,
                EditGestureKind::Delete => config.delete_selection_mode,
            };
            if selection_mode != ConfigSelectionMode::Point {
                if let Some(cell) = raycast_edit_drag_grid(
                    origin,
                    dir,
                    gesture.start,
                    selection_mode,
                    dir,
                    gesture.plane_normal,
                ) {
                    placement.target = Some(TargetHit {
                        pos: cell,
                        normal: IVec3::ZERO,
                    });
                }
            } else {
                placement.target = raycast_blocks(origin, dir, &world);
            }
        } else {
            placement.target = raycast_blocks(origin, dir, &world);
        }
    } else {
        placement.target = raycast_blocks(origin, dir, &world);
    }

    let Ok((_, mut marker_visibility, _)) = marker.single_mut() else {
        return;
    };
    *marker_visibility = Visibility::Hidden;

    let Ok((mut face_transform, mut face_visibility)) = aim_face.single_mut() else {
        return;
    };
    if let Some(target) = placement.target {
        *face_transform = block_face_highlight_transform(target.pos, target.normal);
        *face_visibility = Visibility::Visible;
    } else {
        *face_visibility = Visibility::Hidden;
    }

    if placement.edit_gesture.is_none() {
        hover_bounds.bounds = placement.target.and_then(|target| {
            hover_structure_bounds(&world, &structure_state, debug.factory_activity, target.pos)
        });
    } else {
        hover_bounds.bounds = None;
    }

    if placement.edit_gesture.is_none() {
        despawn_edit_previews(&mut preview_deps.commands, &preview_deps.edit_previews);
        if let (Some(target), Some(block)) = (
            placement
                .target
                .filter(|target| target.normal != IVec3::ZERO),
            selected_place_block(
                &preview_deps.inventory,
                *preview_deps.builder_mode,
                &placement,
            ),
        ) {
            if let Some(render_assets) = preview_deps.render_assets.as_ref() {
                let place_at = target.pos + target.normal;
                let player_pos = preview_deps
                    .player
                    .single()
                    .ok()
                    .map(|transform| transform.translation);
                if can_place_block_at(
                    place_at,
                    block,
                    *preview_deps.builder_mode,
                    &world,
                    player_pos,
                ) {
                    let preview_world = preview_world(&world, &[place_at], block);
                    spawn_block_preview(
                        &mut preview_deps.commands,
                        &mut preview_deps.meshes,
                        render_assets,
                        &preview_world,
                        place_at,
                        block,
                    );
                }
            }
        }
    }
}

fn hover_structure_bounds(
    world: &WorldBlocks,
    structure_state: &StructureState,
    debug_factory: bool,
    pos: IVec3,
) -> Option<StructureBounds> {
    let block = world.blocks.get(&pos)?;
    if block.kind.is_scene() {
        return None;
    }
    if block.kind.is_material() {
        let positions = structure_state
            .pushable_structure_at(pos, IVec3::ZERO)
            .unwrap_or_else(|| material_structure(world, pos));
        return structure_bounds(StructureKind::Material, positions.into_iter());
    }
    if block.kind.is_factory() {
        if !debug_factory {
            return None;
        }
        if structure_state.freedom_at(pos) == Some(StructureFreedom::None) {
            return None;
        }
        let positions = structure_state
            .movable_structure_at(pos)
            .or_else(|| query_factory_structure(world, pos))?;
        return structure_bounds(StructureKind::Factory, positions.into_iter());
    }
    None
}

fn structure_bounds(
    kind: StructureKind,
    mut positions: impl Iterator<Item = IVec3>,
) -> Option<StructureBounds> {
    let first = positions.next()?;
    let mut min = first;
    let mut max = first;
    for pos in positions {
        min = IVec3::new(min.x.min(pos.x), min.y.min(pos.y), min.z.min(pos.z));
        max = IVec3::new(max.x.max(pos.x), max.y.max(pos.y), max.z.max(pos.z));
    }
    Some(StructureBounds { kind, min, max })
}

pub fn draw_hover_structure_bounds(bounds: Res<HoverStructureBounds>, mut gizmos: Gizmos) {
    let Some(bounds) = bounds.bounds else {
        return;
    };
    let min = bounds.min;
    let max = bounds.max;
    let center = (grid_to_world(min) + grid_to_world(max)) * 0.5;
    let size = (max - min + IVec3::ONE).as_vec3() + Vec3::splat(0.06);
    let color = match bounds.kind {
        StructureKind::Material => Color::srgba(1.0, 1.0, 1.0, 0.95),
        StructureKind::Factory => Color::srgba(0.35, 1.0, 0.45, 0.95),
    };
    gizmos.cube(Transform::from_translation(center).with_scale(size), color);
}

fn can_place_in_mode(kind: BlockKind, mode: BuilderMode) -> bool {
    match mode {
        BuilderMode::Edit => kind.is_editable(),
        BuilderMode::Play => kind.is_factory(),
    }
}

fn can_delete_at(pos: IVec3, mode: BuilderMode, world: &WorldBlocks) -> bool {
    match mode {
        BuilderMode::Edit => world.is_occupied(pos),
        BuilderMode::Play => world
            .blocks
            .get(&pos)
            .is_some_and(|block| block.kind.is_factory()),
    }
}

fn delete_block_at(pos: IVec3, mode: BuilderMode, world: &mut WorldBlocks) -> bool {
    match mode {
        BuilderMode::Edit => world.remove(&pos).is_some() || world.remove_system(&pos).is_some(),
        BuilderMode::Play => {
            if !world
                .blocks
                .get(&pos)
                .is_some_and(|block| block.kind.is_factory())
            {
                return false;
            }
            world.remove(&pos).is_some()
        }
    }
}
