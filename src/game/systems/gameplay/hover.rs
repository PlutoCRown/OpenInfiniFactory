//! 悬停准星、结构包围盒与 FOV

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::game::player::controller::FlyCamera;
use crate::game::simulation::structure_state::{
    StructureFreedom, StructureId, StructureKind, StructureState, material_structure,
    query_factory_structure,
};
use crate::game::state::{
    BuilderMode, EditGestureKind, GameMode, GameSettings, PlacementState, PlayingUiState,
    SimulationState, SolutionState,
};
use crate::game::systems::debug::DebugState;
use crate::game::ui::{InventoryItems, UiRuntime};
use crate::game::world::grid::{
    TargetHit, WorldBlocks, grid_to_world, raycast_blocks, raycast_edit_drag_grid,
};
use crate::game::world::rendering::StructureBounds;
use crate::game::world::rendering::{
    AimFaceHighlight, EditPreview, FactoryDebugOverlay, HoverMarker, HoverStructureBounds,
    WorldRenderAssets, block_face_highlight_transform, despawn_edit_previews,
    factory_debug_overlay_material, light_panel_transform, spawn_block_preview,
    spawn_factory_debug_overlay,
};
use crate::scene::BlockEntityIndex;
use crate::shared::config::{ConfigSelectionMode, GameConfig};

use super::placement::{attachment_place_block, preview_world, selected_place_block};
use super::rules::can_place_block_at;

/// 根据设置同步玩家相机 FOV
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

/// 悬停放置预览所需的依赖集合
#[derive(SystemParam)]
pub struct HoverPreviewDeps<'w, 's> {
    commands: Commands<'w, 's>,
    meshes: ResMut<'w, Assets<Mesh>>,
    render_assets: Option<Res<'w, WorldRenderAssets>>,
    inventory: Res<'w, InventoryItems>,
    builder_mode: Res<'w, BuilderMode>,
    solution_state: Res<'w, SolutionState>,
    player: Query<'w, 's, &'static Transform, With<FlyCamera>>,
    edit_previews: Query<'w, 's, Entity, With<EditPreview>>,
}

/// 更新准星目标、面高亮与放置悬停预览
pub fn update_hover(
    mut placement: ResMut<PlacementState>,
    config: Res<GameConfig>,
    mode: Res<State<GameMode>>,
    playing_ui: Res<PlayingUiState>,
    ui_runtime: Res<UiRuntime>,
    simulation: Res<SimulationState>,
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
    } else if let Some(drag) = placement.selection.drag {
        if let Some(cell) =
            raycast_edit_drag_grid(origin, dir, drag.start, ConfigSelectionMode::Line, IVec3::Y)
        {
            placement.target = Some(TargetHit {
                pos: cell,
                normal: IVec3::ZERO,
            });
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
    // 模拟期不显示瞄准面高亮与放置预览（仍保留 target 供传送等）
    if simulation.is_active() {
        *face_visibility = Visibility::Hidden;
        hover_bounds.bounds = None;
        despawn_edit_previews(&mut preview_deps.commands, &preview_deps.edit_previews);
        return;
    }
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
        let light_panel_selected = preview_deps.inventory.hotbar[placement.selected]
            .is_some_and(|item| item.is_light_panel());
        if light_panel_selected {
            if let (Some(target), Some(render_assets)) = (
                placement
                    .target
                    .filter(|target| target.normal != IVec3::ZERO),
                preview_deps.render_assets.as_ref(),
            ) {
                if world.blocks.get(&target.pos).is_some_and(|block| {
                    block.kind.signal_behavior(block.facing)
                        == Some(crate::game::blocks::SignalBehavior::Wire)
                }) {
                    let mut transform = light_panel_transform(target.normal);
                    transform.translation += grid_to_world(target.pos);
                    preview_deps.commands.spawn((
                        Mesh3d(render_assets.light_panel_mesh()),
                        MeshMaterial3d(render_assets.light_panel_material.clone()),
                        transform,
                        EditPreview,
                    ));
                }
            }
        } else if let (Some(target), Some(block)) = (
            placement
                .target
                .filter(|target| target.normal != IVec3::ZERO),
            selected_place_block(
                &preview_deps.inventory,
                *preview_deps.builder_mode,
                preview_deps.solution_state.entry,
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
                    preview_deps.solution_state.entry,
                    &world,
                    player_pos,
                    Some(target.normal),
                ) {
                    let block = attachment_place_block(block, target.normal);
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

/// 计算悬停位置所属结构的包围盒
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

/// 由一组格子位置生成结构包围盒
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

/// 用 gizmos 绘制悬停结构包围盒
pub fn draw_hover_structure_bounds(bounds: Res<HoverStructureBounds>, mut gizmos: Gizmos) {
    let Some(bounds) = bounds.bounds else {
        return;
    };
    let min = bounds.min;
    let max = bounds.max;
    let center = (grid_to_world(min) + grid_to_world(max)) * 0.5;
    let size = (max - min + IVec3::ONE).as_vec3();
    let color = match bounds.kind {
        StructureKind::Material => Color::srgba(1.0, 1.0, 1.0, 0.95),
        StructureKind::Factory => Color::srgba(0.35, 1.0, 0.45, 0.95),
    };
    gizmos.cube(Transform::from_translation(center).with_scale(size), color);
}

/// P 键调试：只给鼠标指向的工厂结构挂活动/固定叠层
pub fn sync_factory_activity_debug_overlays(
    debug: Res<DebugState>,
    placement: Res<PlacementState>,
    world: Res<WorldBlocks>,
    structure_state: Res<StructureState>,
    render_assets: Option<Res<WorldRenderAssets>>,
    block_index: Res<BlockEntityIndex>,
    mut commands: Commands,
    overlays: Query<Entity, With<FactoryDebugOverlay>>,
    mut last: Local<Option<(StructureId, usize)>>,
) {
    let aimed = debug
        .factory_activity
        .then(|| placement.target.as_ref())
        .flatten()
        .and_then(|hit| {
            let block = world.blocks.get(&hit.pos)?;
            if !block.kind.is_factory() {
                return None;
            }
            let id = structure_state.structure_id_at(hit.pos)?;
            let len = structure_state.structure_positions(id)?.len();
            Some((id, len))
        });

    if *last == aimed && !(aimed.is_some() && structure_state.is_changed()) {
        return;
    }

    for entity in &overlays {
        commands.entity(entity).despawn();
    }
    *last = aimed;

    let Some((id, _)) = aimed else {
        return;
    };
    let Some(assets) = render_assets.as_ref() else {
        return;
    };
    let Some(positions) = structure_state.structure_positions(id) else {
        return;
    };
    for &pos in positions {
        let Some(block) = world.blocks.get(&pos) else {
            continue;
        };
        let Some(material) =
            factory_debug_overlay_material(assets, &structure_state, pos, block.kind)
        else {
            continue;
        };
        let Some(entity) = block_index.get_animatable(pos) else {
            continue;
        };
        commands.entity(entity).with_children(|parent| {
            spawn_factory_debug_overlay(parent, assets, material);
        });
    }
}
