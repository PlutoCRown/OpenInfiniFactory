use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use super::components::{
    BlockEntity, BlockEntityLayer, BlockIconRenderEntity, BlockIconRenderRoot, EditPreview,
    FactoryDebugOverlay, PendingGeneratedPreview,
};
use super::connectors::{
    face_mark_transform, local_connector_offset, signal_neighbor_offsets,
    weld_neighbor_connects_to, wire_connects_to,
};
use super::scene_mesh::scene_block_mesh;
use crate::game::blocks::BlockPresent;
use crate::game::blocks::{
    BlockData, BlockKind, BlockModel, WeldConnectorBehavior, WireConnectorBehavior,
    spawn_factory_wire_arm, spawn_model_parts,
};
use crate::game::simulation::structure_state::{FactoryActivity, StructureState};
use crate::game::world::animation::{
    AnimatedBlock, AnimationEasing, AnimationTiming, BlockAnimation, BlockAnimationKind,
    PusherAnimation, rotate_world_pos_y,
};
use crate::game::world::grid::{MaterialFace, WorldBlocks, grid_to_world};
use crate::game::world::render_assets::WorldRenderAssets;
use crate::scene::BlockEntityIndex;

/// 按通电状态选择方块渲染材质
pub(crate) fn block_render_material(
    assets: &WorldRenderAssets,
    data: BlockData,
    powered_wire: bool,
) -> Handle<StandardMaterial> {
    if powered_wire && data.kind == BlockKind::Wire {
        assets.active_wire_material.clone()
    } else {
        assets.block_material(data.kind)
    }
}

/// 按朝向得到方块渲染旋转
fn render_rotation(data: BlockData, facing: crate::game::world::direction::Facing) -> Quat {
    if data.kind.is_directional() {
        Quat::from_rotation_y(facing.yaw())
    } else {
        Quat::IDENTITY
    }
}

/// 世界法线 → 方块局部法线（有向块的面片挂在已 yaw 的实体下）
fn face_mark_local_normal(data: BlockData, world_normal: IVec3) -> IVec3 {
    if data.kind.is_directional() {
        data.facing.inverse_rotate_offset(world_normal)
    } else {
        world_normal
    }
}

/// 工厂调试叠层用材质（活动/未活动）
fn factory_debug_overlay_material(
    assets: &WorldRenderAssets,
    structure_state: &StructureState,
    pos: IVec3,
    kind: BlockKind,
) -> Option<Handle<StandardMaterial>> {
    if !kind.is_factory() {
        return None;
    }
    Some(match structure_state.activity_at(pos) {
        Some(FactoryActivity::Inactive) => assets.inactive_factory_debug_material(),
        _ => assets.active_factory_debug_material(),
    })
}

/// 在方块子节点上挂工厂调试半透明壳
fn spawn_factory_debug_overlay(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderAssets,
    material: Handle<StandardMaterial>,
) {
    parent.spawn((
        Mesh3d(assets.block.clone()),
        MeshMaterial3d(material),
        Transform::from_scale(Vec3::splat(1.03)),
        FactoryDebugOverlay,
        Pickable::IGNORE,
    ));
}

/// 验收预览用：4 个水平面槽位（北→东→南→西）
const GOAL_PREVIEW_FACES: [IVec3; 4] = [
    IVec3::new(0, 0, -1),
    IVec3::X,
    IVec3::new(0, 0, 1),
    IVec3::NEG_X,
];

/// 在生成器/目标上挂所选材料的小预览块
fn spawn_selected_material_preview(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderAssets,
    material: crate::game::blocks::MaterialBlockId,
    facing: crate::game::world::direction::Facing,
    stamps: [Option<crate::game::blocks::StampMaterialId>; 4],
    paints: [Option<crate::game::blocks::PaintMaterialId>; 4],
    icon_render: Option<(Vec3, &RenderLayers)>,
) {
    let kind = BlockKind::material_block_kind(material);
    let rotation = if kind.is_directional() {
        Quat::from_rotation_y(facing.yaw())
    } else {
        Quat::IDENTITY
    };
    let mesh = assets
        .scene_mesh(kind)
        .unwrap_or_else(|| assets.block_mesh(kind));
    let mesh_material = assets
        .scene_material(kind)
        .unwrap_or_else(|| assets.block_material(kind));

    let mut child = parent.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mesh_material),
        Transform {
            rotation,
            scale: Vec3::splat(0.76),
            ..default()
        },
    ));
    if let Some((_, icon_layer)) = icon_render {
        child.insert((icon_layer.clone(), BlockIconRenderEntity));
    }
    child.with_children(|preview| {
        spawn_goal_attachment_previews(
            preview,
            assets,
            stamps,
            paints,
            icon_render.map(|(_, layer)| layer),
        );
    });
}

/// 按槽位把印花/漆挂到材料预览或验收幽灵上（像真实方块附着）
fn spawn_goal_attachment_previews(
    parent: &mut ChildSpawnerCommands,
    assets: &WorldRenderAssets,
    stamps: [Option<crate::game::blocks::StampMaterialId>; 4],
    paints: [Option<crate::game::blocks::PaintMaterialId>; 4],
    icon_layer: Option<&RenderLayers>,
) {
    for (index, face) in GOAL_PREVIEW_FACES.iter().enumerate() {
        if let Some(paint) = paints[index] {
            let mut child = parent.spawn((
                Mesh3d(assets.face_mark_mesh(*face)),
                MeshMaterial3d(assets.face_mark_material(paint)),
                face_mark_transform(*face, 0.05),
            ));
            if let Some(layer) = icon_layer {
                child.insert((layer.clone(), BlockIconRenderEntity));
            }
        }
        if let Some(stamp) = stamps[index] {
            let kind = BlockKind::stamp_block_kind(stamp);
            let mesh = assets
                .scene_mesh(kind)
                .unwrap_or_else(|| assets.block_mesh(kind));
            let mesh_material = assets
                .scene_material(kind)
                .unwrap_or_else(|| assets.block_material(kind));
            let normal = face.as_vec3().normalize_or_zero();
            let mut child = parent.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(mesh_material),
                Transform {
                    translation: normal * 0.5,
                    rotation: Quat::from_rotation_arc(Vec3::Z, normal),
                    ..default()
                },
            ));
            if let Some(layer) = icon_layer {
                child.insert((layer.clone(), BlockIconRenderEntity));
            }
        }
    }
}

/// 无动画生成世界方块实体
pub fn spawn_block(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
    factory_debug: Option<&StructureState>,
    index: &mut BlockEntityIndex,
) {
    spawn_block_with_animation(
        commands,
        meshes,
        assets,
        world,
        pos,
        data,
        None,
        factory_debug,
        index,
    );
}

/// 以编辑时序生成可带动画的方块
pub fn spawn_block_with_animation(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
    animation: Option<BlockAnimation>,
    factory_debug: Option<&StructureState>,
    index: &mut BlockEntityIndex,
) -> Entity {
    spawn_block_with_timed_animation(
        commands,
        meshes,
        assets,
        world,
        pos,
        data,
        animation,
        AnimationTiming::edit(),
        factory_debug,
        false,
        index,
    )
}

/// 以指定时序/通电状态生成方块
pub fn spawn_block_with_timed_animation(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
    animation: Option<BlockAnimation>,
    timing: AnimationTiming,
    factory_debug: Option<&StructureState>,
    powered_wire: bool,
    index: &mut BlockEntityIndex,
) -> Entity {
    spawn_block_model(
        commands,
        meshes,
        assets,
        world,
        pos,
        data,
        block_render_material(assets, data, powered_wire),
        None,
        animation,
        None,
        timing,
        true,
        false,
        true,
        None,
        factory_debug,
        Some(index),
    )
}

/// 生成待落地的半透明生成预览方块
pub fn spawn_pending_generated_block(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
    animation: Option<BlockAnimation>,
    timing: AnimationTiming,
) {
    spawn_block_model(
        commands,
        meshes,
        assets,
        world,
        pos,
        data,
        assets.block_material(data.kind),
        None,
        animation,
        None,
        timing,
        false,
        true,
        false,
        None,
        None,
        None,
    );
}

/// 增量场景用：生成完整世界方块实体
pub(crate) fn spawn_world_block_entity(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    index: &mut BlockEntityIndex,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
    animation: Option<BlockAnimation>,
    pusher_animation: Option<PusherAnimation>,
    timing: AnimationTiming,
    powered_wire: bool,
    factory_debug: Option<&StructureState>,
) -> Entity {
    spawn_block_model(
        commands,
        meshes,
        assets,
        world,
        pos,
        data,
        block_render_material(assets, data, powered_wire),
        None,
        animation,
        pusher_animation,
        timing,
        true,
        false,
        true,
        None,
        factory_debug,
        Some(index),
    )
}

/// 核心：按数据生成方块模型、连接器与子部件
pub(crate) fn spawn_block_model(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
    material: Handle<StandardMaterial>,
    edit_preview: Option<EditPreview>,
    animation: Option<BlockAnimation>,
    pusher_animation: Option<PusherAnimation>,
    timing: AnimationTiming,
    with_block_entity: bool,
    pending_generated_preview: bool,
    show_generator_preview: bool,
    icon_render: Option<(Vec3, &RenderLayers)>,
    factory_debug: Option<&StructureState>,
    index: Option<&mut BlockEntityIndex>,
) -> Entity {
    let debug_overlay = factory_debug.and_then(|structure_state| {
        factory_debug_overlay_material(assets, structure_state, pos, data.kind)
    });
    let mut transform = Transform::from_translation(grid_to_world(pos));
    if let Some(animation) = animation {
        let progress = animation.progress.unwrap_or(0.0).clamp(0.0, 1.0);
        let eased = match timing.easing {
            AnimationEasing::Linear => progress,
            AnimationEasing::SmoothStep => progress * progress * (3.0 - 2.0 * progress),
        };
        transform.translation = match animation.kind {
            BlockAnimationKind::Rotate { pivot, clockwise } => {
                rotate_world_pos_y(grid_to_world(animation.from_pos), pivot, clockwise, eased)
            }
            _ => grid_to_world(animation.from_pos).lerp(grid_to_world(animation.to_pos), eased),
        };
        transform.rotation = render_rotation(data, animation.from_facing)
            .slerp(render_rotation(data, animation.to_facing), eased);
        transform.scale = match animation.kind {
            BlockAnimationKind::Move | BlockAnimationKind::Rotate { .. } => Vec3::ONE,
            BlockAnimationKind::SpawnScale => Vec3::splat(eased),
        };
    } else {
        transform.rotation = if data
            .kind
            .material_props()
            .is_some_and(|props| props.is_stamp)
        {
            // 印花：局部 +Z 朝外（宿主→印花法线）；无 GLB 的薄板已偏向 -Z 贴宿主
            world
                .material_attachments
                .get(&data.id)
                .map(|att| {
                    Quat::from_rotation_arc(Vec3::Z, att.parent_face_normal.as_vec3().normalize())
                })
                .unwrap_or(Quat::IDENTITY)
        } else {
            render_rotation(data, data.facing)
        };
    }
    if let Some((origin, _)) = icon_render {
        transform.translation += origin;
    }

    let shell_scale = data.kind.material_shell_scale();
    if shell_scale != 1.0 {
        transform.scale *= shell_scale;
    }

    let is_preview = edit_preview.is_some();
    let has_factory_visual = assets.factory_visual(data.kind).is_some();
    // 游玩态验收器：半透明目标材料 + 扫光；编辑/图标/放置预览仍用绿壳
    let use_goal_ghost = data.kind == BlockKind::Goal
        && assets.use_goal_play_visual()
        && !is_preview
        && icon_render.is_none();
    let goal_settings = use_goal_ghost.then(|| world.goal_settings(pos));
    if let Some(settings) = goal_settings {
        let mat_kind = BlockKind::material_block_kind(settings.material);
        transform.rotation =
            render_rotation(BlockData::new(mat_kind, settings.facing), settings.facing);
    }
    let goal_material_kind =
        goal_settings.map(|settings| BlockKind::material_block_kind(settings.material));
    let mut entity = if let Some(mat_kind) = goal_material_kind {
        if let Some(ghost) = assets.goal_ghost_material(mat_kind) {
            commands.spawn((
                Mesh3d(assets.goal_ghost_mesh(mat_kind)),
                MeshMaterial3d(ghost),
                transform,
            ))
        } else {
            commands.spawn((
                Mesh3d(assets.block_mesh(data.kind)),
                MeshMaterial3d(material.clone()),
                transform,
            ))
        }
    } else if data.kind == crate::game::blocks::BlockKind::Wire
        || has_factory_visual
        || matches!(
            data.kind.model(),
            BlockModel::PartsOnly(_) | BlockModel::PusherParts(_)
        )
    {
        commands.spawn((transform, Visibility::default()))
    } else if data.kind == BlockKind::Platform {
        commands.spawn((
            Mesh3d(assets.block_mesh(data.kind)),
            MeshMaterial3d(if is_preview {
                assets.model_preview_material(crate::game::blocks::ModelMaterial::Platform)
            } else {
                assets.model_material(crate::game::blocks::ModelMaterial::Platform)
            }),
            transform,
        ))
    } else {
        match assets.scene_material(data.kind) {
            Some(scene_material) => {
                let mesh = if icon_render.is_some() {
                    assets
                        .scene_mesh(data.kind)
                        .unwrap_or_else(|| assets.block_mesh(data.kind))
                } else if let Some(face_uvs) = assets.scene_face_uvs(data.kind) {
                    meshes.add(scene_block_mesh(pos, world, assets, face_uvs))
                } else {
                    assets
                        .scene_mesh(data.kind)
                        .unwrap_or_else(|| assets.block_mesh(data.kind))
                };
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(if is_preview {
                        material.clone()
                    } else {
                        scene_material
                    }),
                    transform,
                ))
            }
            None => commands.spawn((
                Mesh3d(assets.block_mesh(data.kind)),
                MeshMaterial3d(material.clone()),
                transform,
            )),
        }
    };

    if let Some((_, icon_layer)) = icon_render {
        entity.insert((
            icon_layer.clone(),
            BlockIconRenderEntity,
            BlockIconRenderRoot,
        ));
    }

    if with_block_entity {
        let layer = BlockEntityLayer::from_kind(data.kind);
        entity.insert(BlockEntity {
            pos,
            id: data.id,
            layer,
        });
        if let Some(index) = index {
            index.insert(pos, data.id, layer, entity.id());
        } else {
            debug_assert!(false, "with_block_entity 时必须传入 BlockEntityIndex");
        }
    }

    if pending_generated_preview {
        entity.insert(PendingGeneratedPreview);
    }

    if let Some(edit_preview) = edit_preview {
        entity.insert(edit_preview);
    }

    if let Some(animation) = animation.filter(|_| !pending_generated_preview) {
        entity.insert(AnimatedBlock::new(animation, timing));
    }

    entity.with_children(|parent| {
        let mut model_root = parent.spawn((Transform::default(), Visibility::default()));
        if let Some((_, icon_layer)) = icon_render {
            model_root.insert((icon_layer.clone(), BlockIconRenderEntity));
        }
        model_root.with_children(|parent| {
            spawn_model_parts(
                parent,
                assets,
                data.kind,
                data.kind.model(),
                pusher_animation,
                icon_render.map(|(_, layer)| layer),
                is_preview,
            );
        });

        let render_behavior = data.kind.render_behavior(data.facing);

        if let Some(weld_connector) = render_behavior.weld_connector {
            let offsets = match weld_connector {
                WeldConnectorBehavior::AllSides => signal_neighbor_offsets().to_vec(),
                WeldConnectorBehavior::Offset(offset) => vec![offset],
            };
            for offset in offsets {
                let neighbor = pos + offset;
                if weld_neighbor_connects_to(world, neighbor, -offset) {
                    let local_offset = local_connector_offset(data, offset);
                    let translation = local_offset.as_vec3() * 0.225;
                    let mut child = parent.spawn((
                        Mesh3d(assets.connector_mesh(local_offset)),
                        MeshMaterial3d(assets.weld_connector_material.clone()),
                        Transform::from_translation(translation),
                    ));
                    if let Some((_, icon_layer)) = icon_render {
                        child.insert((icon_layer.clone(), BlockIconRenderEntity));
                    }
                }
            }
        }

        if let Some(wire_connector) = render_behavior.wire_connector {
            let blocked_offset = match wire_connector {
                WireConnectorBehavior::Device { blocked_offset } => Some(blocked_offset),
                WireConnectorBehavior::Wire => None,
            };
            let use_factory_wire = data.kind == BlockKind::Wire
                && matches!(
                    assets.factory_visual(BlockKind::Wire),
                    Some(crate::game::world::render_assets::FactoryVisual::Wire { .. })
                );
            let powered_wire = material == assets.active_wire_material;
            let mut connected_offsets = Vec::new();
            for (face_index, offset) in signal_neighbor_offsets().into_iter().enumerate() {
                if blocked_offset == Some(offset) {
                    continue;
                }
                if data.kind == BlockKind::Wire
                    && world
                        .wire_face_panels
                        .contains(&MaterialFace::new(data.id, offset))
                {
                    continue;
                }
                let neighbor = pos + offset;
                let neighbor_block = world
                    .blocks
                    .get(&neighbor)
                    .or_else(|| world.system_blocks.get(&neighbor));
                if neighbor_block.is_some_and(|block| {
                    if block.kind == BlockKind::Wire
                        && world
                            .wire_face_panels
                            .contains(&MaterialFace::new(block.id, -offset))
                    {
                        return false;
                    }
                    wire_connects_to(block, -offset)
                }) {
                    connected_offsets.push((face_index, offset));
                    if !use_factory_wire {
                        let local_offset = local_connector_offset(data, offset);
                        let mut child = parent.spawn((
                            Mesh3d(assets.wire_connector_mesh(local_offset)),
                            MeshMaterial3d(if data.kind == BlockKind::Wire {
                                material.clone()
                            } else {
                                assets.wire_connector_material.clone()
                            }),
                            Transform::from_translation(local_offset.as_vec3() * 0.174),
                        ));
                        if let Some((_, icon_layer)) = icon_render {
                            child.insert((icon_layer.clone(), BlockIconRenderEntity));
                        }
                    }
                }
            }

            if use_factory_wire {
                // 孤立或图标：六向全画；单边连通时补对向臂（露出金属环端面）
                let mut faces: Vec<usize> = if connected_offsets.is_empty() || icon_render.is_some()
                {
                    (0..6).collect()
                } else {
                    connected_offsets.iter().map(|(i, _)| *i).collect()
                };
                if connected_offsets.len() == 1 {
                    let only = connected_offsets[0].0;
                    let opposite = only ^ 1;
                    if !faces.contains(&opposite) {
                        faces.push(opposite);
                    }
                }
                for face_index in faces {
                    spawn_factory_wire_arm(
                        parent,
                        assets,
                        face_index,
                        icon_render.map(|(_, layer)| layer),
                        is_preview,
                        powered_wire,
                    );
                }
            } else if data.kind == crate::game::blocks::BlockKind::Wire
                && connected_offsets.is_empty()
            {
                let mut child = parent.spawn((
                    Mesh3d(assets.wire_node_mesh()),
                    MeshMaterial3d(material.clone()),
                ));
                if let Some((_, icon_layer)) = icon_render {
                    child.insert((icon_layer.clone(), BlockIconRenderEntity));
                }
            }

            if data.kind == BlockKind::Wire {
                let panel_lit = material == assets.active_wire_material;
                for face in world
                    .wire_face_panels
                    .iter()
                    .filter(|face| face.block == data.id)
                {
                    let panel_material = if panel_lit {
                        assets.light_panel_lit_material.clone()
                    } else {
                        assets.light_panel_material.clone()
                    };
                    let mut child = parent.spawn((
                        Mesh3d(assets.face_mark_mesh(face.normal)),
                        MeshMaterial3d(panel_material),
                        face_mark_transform(face.normal, 0.01),
                    ));
                    if let Some((_, icon_layer)) = icon_render {
                        child.insert((icon_layer.clone(), BlockIconRenderEntity));
                    }
                }
            }
        }

        if data.kind.is_material() {
            for (face, paint) in world
                .material_paints
                .iter()
                .filter(|(face, _)| face.block == data.id)
            {
                // 局部法线贴在真实附着面；有向块的 yaw 由父实体承担
                let local_normal = face_mark_local_normal(data, face.normal);
                let mut child = parent.spawn((
                    Mesh3d(assets.face_mark_mesh(local_normal)),
                    MeshMaterial3d(assets.face_mark_material(*paint)),
                    face_mark_transform(local_normal, 0.05),
                ));
                if let Some((_, icon_layer)) = icon_render {
                    child.insert((icon_layer.clone(), BlockIconRenderEntity));
                }
            }
        }

        if use_goal_ghost {
            let settings = world.goal_settings(pos);
            spawn_goal_attachment_previews(
                parent,
                assets,
                settings.stamps,
                settings.paints,
                icon_render.map(|(_, layer)| layer),
            );
        }

        if show_generator_preview {
            let show_material_preview = match data.kind {
                // 游玩/编辑显示配置材料；模拟回合重建时 show_generator_preview=false
                BlockKind::Generator => true,
                // 游玩态验收器本体已是目标材料，不再挂小预览
                BlockKind::Goal => !use_goal_ghost,
                _ => false,
            };
            if show_material_preview {
                if data.kind == BlockKind::Generator {
                    let settings = world.generator_settings(pos);
                    spawn_selected_material_preview(
                        parent,
                        assets,
                        settings.material,
                        settings.facing,
                        [None; 4],
                        [None; 4],
                        icon_render,
                    );
                } else {
                    let settings = world.goal_settings(pos);
                    spawn_selected_material_preview(
                        parent,
                        assets,
                        settings.material,
                        settings.facing,
                        settings.stamps,
                        settings.paints,
                        icon_render,
                    );
                }
            }
        }

        if let Some(material) = debug_overlay {
            spawn_factory_debug_overlay(parent, assets, material);
        }
    });
    entity.id()
}
