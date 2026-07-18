//! 放置/删除手势、预览与传送

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::HashSet;

use crate::game::blocks::BlockPresent;
use crate::game::blocks::{BlockData, BlockKind};
use crate::game::edit_history::{build_cell_patch, EditHistory, FacePanelDelta, WorldPatch};
use crate::game::player::controller::{teleport_player_preserve_offset, FlyCamera};
use crate::game::simulation::markers::refresh_static_generated_markers;
use crate::game::simulation::structure_state::StructureState;
use crate::game::state::{
    BuilderMode, EditGesture, EditGestureKind, GameMode, PlacementState, PlayingUiState,
    SelectionBounds, SimulationState, SolutionState,
};
use crate::game::systems::debug::DebugState;
use crate::game::ui::{AreaKind, InventoryItems, UiRuntime};
use crate::game::world::direction::Facing;
use crate::game::world::grid::{MaterialFace, WorldBlocks};
use crate::game::world::rendering::{
    despawn_edit_previews, spawn_block_preview, spawn_delete_bounds_preview, BlockEntity,
    EditPreview, WorldRenderAssets,
};
use crate::scene::{refresh_edit_changes, BlockEntityIndex};
use crate::shared::config::{ConfigSelectionMode, GameConfig};

use super::edit_ops::{
    alternate_block_at, pick_target_block, rotate_block_at, rotate_facing, shift_pressed,
};
use super::rules::{can_delete_at, can_place_block_at, can_place_in_mode, delete_block_at};
use super::selection::{
    handle_selection_area_input, selection_positions, spawn_selection_previews,
};

/// 放置输入所需的查询与资源集合
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
    input: Res<'w, crate::game::input::GameplayInputState>,
    touch: Res<'w, crate::shared::touch_profile::TouchProfile>,
}

/// 处理放置/删除手势、取块、旋转与框选入口
pub fn placement_input(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut world: ResMut<WorldBlocks>,
    mut solution_state: ResMut<SolutionState>,
    mut edit_history: ResMut<EditHistory>,
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
        input,
        touch,
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

    if input.cancel_edit_gesture {
        if let Some(gesture) = placement.edit_gesture.as_mut() {
            gesture.canceled = true;
        }
    }

    if input.open_block_config {
        let current_target_pos = placement.target.map(|target| target.pos);
        if open_target_block_ui(current_target_pos, &world, *builder_mode, &mut ui_runtime) {
            placement.edit_gesture = None;
            placement.selection.clear();
            despawn_edit_previews(&mut commands, &edit_previews);
            return;
        }
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
    // 触控下配置走独立按钮，放置键不再误开面板
    if input.place.just_pressed
        && !force_place
        && !touch.enabled
        && open_target_block_ui(current_target_pos, &world, *builder_mode, &mut ui_runtime)
    {
        placement.edit_gesture = None;
        placement.selection.clear();
        despawn_edit_previews(&mut commands, &edit_previews);
        return;
    }

    if selected_area(&inventory, &placement) == Some(AreaKind::Selection) {
        let copy_chord = config.chord(crate::shared::config::ActionKeyName::Copy);
        if handle_selection_area_input(
            &mouse_buttons,
            &keys,
            current_target_pos,
            place_button,
            delete_button,
            copy_chord,
            force_place,
            *builder_mode,
            &mut placement,
            &mut world,
            &mut edit_history,
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
        spawn_selection_previews(
            &placement,
            &world,
            *builder_mode,
            force_place,
            &mut commands,
            &render_assets,
        );
        return;
    }

    if placement.selection.is_active() {
        placement.selection.clear();
    }

    if input.pick.just_pressed {
        if let Some(pos) = current_target_pos {
            pick_target_block(pos, &world, &mut placement, &mut inventory);
        }
        placement.edit_gesture = None;
        despawn_edit_previews(&mut commands, &edit_previews);
        return;
    }

    if *builder_mode == BuilderMode::Play && !simulation.is_active() && input.alternate {
        if let Some(pos) = current_target_pos {
            edit_history.flush_pending_rotation();
            if alternate_block_at(
                pos,
                &mut world,
                &mut edit_history,
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

    if input.rotate {
        let reverse_rotation = shift_pressed(&keys);
        if let Some(gesture) = placement.edit_gesture.as_mut() {
            if let EditGestureKind::Place { block } = &mut gesture.kind {
                if block.kind.is_directional() {
                    block.facing = rotate_facing(block.facing, reverse_rotation);
                    placement.preview_facing = block.facing;
                }
            }
        } else if let Some(pos) = current_target_pos {
            edit_history.prepare_rotation(&world, pos);
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
                let facing = world
                    .blocks
                    .get(&pos)
                    .or_else(|| world.system_blocks.get(&pos))
                    .map(|block| block.facing);
                if let Some(facing) = facing {
                    edit_history.finish_rotation(pos, facing);
                }
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

    if input.delete.just_pressed {
        if *builder_mode == BuilderMode::Play
            && try_player_teleport(current_target_pos, &world, &mut player)
        {
            placement.edit_gesture = None;
            despawn_edit_previews(&mut commands, &edit_previews);
            return;
        }
        // 优先卸下瞄准面的灯面板（不占格，点对面删除）
        if let Some(target) = placement
            .target
            .filter(|target| target.normal != IVec3::ZERO)
        {
            if let Some(block) = world.blocks.get(&target.pos).copied() {
                if block.kind == BlockKind::Wire {
                    let face = MaterialFace::new(block.id, target.normal);
                    if world.wire_face_panels.contains(&face) {
                        edit_history.flush_pending_rotation();
                        let patch = WorldPatch {
                            face_panels: vec![FacePanelDelta {
                                pos: target.pos,
                                face,
                                before: true,
                                after: false,
                            }],
                            ..Default::default()
                        };
                        patch.apply_forward(&mut world);
                        edit_history.record(patch);
                        refresh_static_generated_markers(&mut world);
                        refresh_edit_changes(
                            &mut commands,
                            &mut meshes,
                            &mut block_index,
                            &world,
                            &render_assets,
                            &debug,
                            &mut structure_state,
                            &HashSet::from([
                                target.pos,
                                target.pos + IVec3::X,
                                target.pos + IVec3::NEG_X,
                                target.pos + IVec3::Y,
                                target.pos + IVec3::NEG_Y,
                                target.pos + IVec3::Z,
                                target.pos + IVec3::NEG_Z,
                            ]),
                        );
                        solution_state.dirty = true;
                        placement.edit_gesture = None;
                        despawn_edit_previews(&mut commands, &edit_previews);
                        return;
                    }
                }
            }
        }
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

    if input.place.just_pressed {
        // 灯面板：点在电线表面即时粘贴，不走占格手势
        if inventory.hotbar[placement.selected].is_some_and(|item| item.is_light_panel()) {
            if let Some(target) = placement
                .target
                .filter(|target| target.normal != IVec3::ZERO)
            {
                if let Some(block) = world.blocks.get(&target.pos).copied() {
                    if block.kind == BlockKind::Wire {
                        let face = MaterialFace::new(block.id, target.normal);
                        if !world.wire_face_panels.contains(&face) {
                            edit_history.flush_pending_rotation();
                            let patch = WorldPatch {
                                face_panels: vec![FacePanelDelta {
                                    pos: target.pos,
                                    face,
                                    before: false,
                                    after: true,
                                }],
                                ..Default::default()
                            };
                            patch.apply_forward(&mut world);
                            edit_history.record(patch);
                            refresh_edit_changes(
                                &mut commands,
                                &mut meshes,
                                &mut block_index,
                                &world,
                                &render_assets,
                                &debug,
                                &mut structure_state,
                                &HashSet::from([
                                    target.pos,
                                    target.pos + IVec3::X,
                                    target.pos + IVec3::NEG_X,
                                    target.pos + IVec3::Y,
                                    target.pos + IVec3::NEG_Y,
                                    target.pos + IVec3::Z,
                                    target.pos + IVec3::NEG_Z,
                                ]),
                            );
                            solution_state.dirty = true;
                        }
                    }
                }
            }
            placement.edit_gesture = None;
            despawn_edit_previews(&mut commands, &edit_previews);
            return;
        }
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

    let released_place = input.place.just_released;
    let released_delete = input.delete.just_released;
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
                    &mut edit_history,
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

    let _ = (place_button, delete_button, pick_button);
}

/// 取当前快捷栏选中的可放置方块
pub(super) fn selected_place_block(
    inventory: &InventoryItems,
    builder_mode: BuilderMode,
    placement: &PlacementState,
) -> Option<BlockData> {
    let kind = inventory.hotbar[placement.selected]?;
    let kind = kind.block()?;
    can_place_in_mode(kind, builder_mode).then_some(BlockData::new(kind, placement.preview_facing))
}

/// 取当前快捷栏选中的区域工具种类
fn selected_area(inventory: &InventoryItems, placement: &PlacementState) -> Option<AreaKind> {
    inventory.hotbar[placement.selected].and_then(|item| item.area())
}

/// 右键传送入口/出口时传送玩家
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

/// 点击可配置方块时打开方块 UI（编辑态系统块，或玩法/编辑态告示等）
fn open_target_block_ui(
    target: Option<IVec3>,
    world: &WorldBlocks,
    builder_mode: BuilderMode,
    ui_runtime: &mut UiRuntime,
) -> bool {
    let Some(pos) = target else {
        return false;
    };
    let block = match builder_mode {
        BuilderMode::Edit => world
            .system_blocks
            .get(&pos)
            .or_else(|| world.blocks.get(&pos)),
        BuilderMode::Play => world.blocks.get(&pos),
    };
    let Some(block) = block else {
        return false;
    };

    let Some(panel) = block.kind.ui_panel() else {
        return false;
    };

    ui_runtime.open_block(panel, pos);
    true
}

/// 清除全部方块实体并重置索引
pub(super) fn despawn_block_entities(
    commands: &mut Commands,
    block_entities: &Query<(Entity, &BlockEntity)>,
    block_index: &mut BlockEntityIndex,
) {
    block_index.clear();
    for (entity, _) in block_entities {
        commands.entity(entity).despawn();
    }
}

/// 编辑后刷新静态生成标记
pub(super) fn refresh_edit_generated_markers(world: &mut WorldBlocks) {
    refresh_static_generated_markers(world);
}

/// 提交放置/删除手势并写入历史
fn commit_edit_gesture(
    gesture: EditGesture,
    current_place_at: Option<IVec3>,
    current_delete_at: Option<IVec3>,
    config: &GameConfig,
    world: &mut WorldBlocks,
    builder_mode: BuilderMode,
    player_pos: Option<Vec3>,
    edit_history: &mut EditHistory,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    render_assets: &WorldRenderAssets,
    debug: &DebugState,
    structure_state: &mut StructureState,
    block_index: &mut BlockEntityIndex,
) -> bool {
    let patch = match gesture.kind {
        EditGestureKind::Place { block } => {
            let positions = selection_positions(
                config.place_selection_mode,
                gesture.start,
                current_place_at.unwrap_or(gesture.start),
            );
            let positions: Vec<IVec3> = positions
                .into_iter()
                .filter(|pos| {
                    can_place_block_at(
                        *pos,
                        block,
                        builder_mode,
                        world,
                        player_pos,
                        Some(gesture.plane_normal),
                    )
                })
                .collect();
            if positions.is_empty() {
                return false;
            }
            build_cell_patch(world, &positions, |world| {
                for pos in &positions {
                    let mut placed = block;
                    // 侧贴：朝向取贴面法线；顶立：保留预览朝向
                    if block.kind == BlockKind::Sign {
                        let normal = gesture.plane_normal;
                        if normal.y == 0 {
                            placed.facing = match (normal.x, normal.z) {
                                (1, 0) => Facing::East,
                                (-1, 0) => Facing::West,
                                (0, 1) => Facing::South,
                                (0, -1) => Facing::North,
                                _ => placed.facing,
                            };
                        }
                    }
                    world.insert(*pos, placed);
                    if block.kind == BlockKind::Sign {
                        let host_pos = *pos - gesture.plane_normal;
                        if let Some(host) = world.blocks.get(&host_pos).copied() {
                            if let Some(sign) = world.blocks.get(pos).copied() {
                                world.attach_factory_child(sign.id, host.id, gesture.plane_normal);
                            }
                        }
                    }
                }
            })
        }
        EditGestureKind::Delete => {
            let positions = selection_positions(
                config.delete_selection_mode,
                gesture.start,
                current_delete_at.unwrap_or(gesture.start),
            );
            let positions: Vec<IVec3> = positions
                .into_iter()
                .filter(|pos| can_delete_at(*pos, builder_mode, world))
                .collect();
            if positions.is_empty() {
                return false;
            }
            build_cell_patch(world, &positions, |world| {
                for pos in &positions {
                    delete_block_at(*pos, builder_mode, world);
                }
            })
        }
    };
    if patch.is_empty() {
        return false;
    }
    let changed_positions = patch.affected_positions();
    edit_history.record(patch);
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

/// 为进行中的放置/删除手势生成预览
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
                .filter(|pos| {
                    can_place_block_at(
                        *pos,
                        block,
                        builder_mode,
                        world,
                        player_pos,
                        Some(gesture.plane_normal),
                    )
                })
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

/// 克隆世界并插入预览方块，用于连通预览渲染
pub(super) fn preview_world(
    world: &WorldBlocks,
    positions: &[IVec3],
    block: BlockData,
) -> WorldBlocks {
    let mut preview = world.clone();
    for pos in positions {
        preview.insert(*pos, block);
    }
    refresh_edit_generated_markers(&mut preview);
    preview
}
