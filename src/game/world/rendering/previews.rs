use bevy::prelude::*;

use super::components::{
    DeleteBoundsOverlay, DeleteBoundsPart, EditPreview, GameplayScene, PendingGeneratedPreview,
    SelectionBoundsOverlay, SelectionBoundsPart,
};
use super::spawn::spawn_block_model;
use crate::game::blocks::BlockData;
use crate::game::world::animation::AnimationTiming;
use crate::game::world::grid::{WorldBlocks, grid_to_world};
use crate::game::world::render_assets::{EditPreviewKind, WorldRenderAssets};

/// 包围盒边线厚度（世界单位）
const BOUNDS_EDGE_T: f32 = 0.017;
/// 包围盒角块边长（世界单位）
const BOUNDS_CORNER_S: f32 = 0.044;

/// 清除所有编辑预览实体
pub fn despawn_edit_previews(commands: &mut Commands, previews: &Query<Entity, With<EditPreview>>) {
    for entity in previews {
        commands.entity(entity).despawn();
    }
}

/// 清除所有待生成预览实体
pub fn despawn_pending_generated_previews(
    commands: &mut Commands,
    previews: &Query<Entity, With<PendingGeneratedPreview>>,
) {
    for entity in previews {
        commands.entity(entity).despawn();
    }
}

/// 生成单格编辑预览（放置/删除色块）
pub fn spawn_edit_preview(
    commands: &mut Commands,
    assets: &WorldRenderAssets,
    pos: IVec3,
    kind: EditPreviewKind,
) {
    commands.spawn((
        Mesh3d(assets.block.clone()),
        MeshMaterial3d(assets.edit_preview_material(kind)),
        Transform::from_translation(grid_to_world(pos)).with_scale(Vec3::splat(1.03)),
        EditPreview,
    ));
}

/// 场景启动时创建全局唯一的选区/删除包围盒实体
pub fn spawn_bounds_overlays(commands: &mut Commands, assets: &WorldRenderAssets) {
    let mesh = assets.block.clone();
    let fill = assets.selection_fill_material();
    let edge = assets.selection_edge_material();
    let delete_fill = assets.selection_invalid_fill_material();
    let delete_edge = assets.selection_invalid_edge_material();

    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(fill),
        Transform::default(),
        Visibility::Hidden,
        SelectionBoundsOverlay,
        SelectionBoundsPart::Fill,
        GameplayScene,
    ));
    for i in 0..12u8 {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(edge.clone()),
            Transform::default(),
            Visibility::Hidden,
            SelectionBoundsOverlay,
            SelectionBoundsPart::Edge(i),
            GameplayScene,
        ));
    }
    for i in 0..8u8 {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(edge.clone()),
            Transform::default(),
            Visibility::Hidden,
            SelectionBoundsOverlay,
            SelectionBoundsPart::Corner(i),
            GameplayScene,
        ));
    }

    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(delete_fill),
        Transform::default(),
        Visibility::Hidden,
        DeleteBoundsOverlay,
        DeleteBoundsPart::Fill,
        GameplayScene,
    ));
    for i in 0..12u8 {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(delete_edge.clone()),
            Transform::default(),
            Visibility::Hidden,
            DeleteBoundsOverlay,
            DeleteBoundsPart::Edge(i),
            GameplayScene,
        ));
    }
}

/// 更新全局选区包围盒；None 则隐藏
pub fn update_selection_bounds_overlay(
    parts: &mut Query<
        (
            &SelectionBoundsPart,
            &mut Transform,
            &mut Visibility,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        (With<SelectionBoundsOverlay>, Without<DeleteBoundsOverlay>),
    >,
    assets: &WorldRenderAssets,
    show: Option<(IVec3, IVec3, bool, bool)>,
) {
    let Some((min, max, include_frame, valid)) = show else {
        for (_, _, mut visibility, _) in parts.iter_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    };

    let center = (grid_to_world(min) + grid_to_world(max)) * 0.5;
    let size = (max - min + IVec3::ONE).as_vec3();
    let half = size * 0.5;
    let edges = bounds_edge_transforms(center, size, half);
    let corners = bounds_corner_positions(center, half);
    let fill_mat = if valid {
        assets.selection_fill_material()
    } else {
        assets.selection_invalid_fill_material()
    };
    let edge_mat = if valid {
        assets.selection_edge_material()
    } else {
        assets.selection_invalid_edge_material()
    };

    for (part, mut transform, mut visibility, mut material) in parts.iter_mut() {
        match *part {
            SelectionBoundsPart::Fill => {
                *transform = Transform::from_translation(center).with_scale(size);
                *material = MeshMaterial3d(fill_mat.clone());
                *visibility = Visibility::Visible;
            }
            SelectionBoundsPart::Edge(i) => {
                if include_frame {
                    let (pos, scale) = edges[i as usize];
                    *transform = Transform::from_translation(pos).with_scale(scale);
                    *material = MeshMaterial3d(edge_mat.clone());
                    *visibility = Visibility::Visible;
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
            SelectionBoundsPart::Corner(i) => {
                if include_frame {
                    *transform = Transform::from_translation(corners[i as usize])
                        .with_scale(Vec3::splat(BOUNDS_CORNER_S));
                    *material = MeshMaterial3d(edge_mat.clone());
                    *visibility = Visibility::Visible;
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
}

/// 更新全局删除包围盒；None 则隐藏
pub fn update_delete_bounds_overlay(
    parts: &mut Query<
        (&DeleteBoundsPart, &mut Transform, &mut Visibility),
        (With<DeleteBoundsOverlay>, Without<SelectionBoundsOverlay>),
    >,
    show: Option<(IVec3, IVec3)>,
) {
    let Some((min, max)) = show else {
        for (_, _, mut visibility) in parts.iter_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    };

    let center = (grid_to_world(min) + grid_to_world(max)) * 0.5;
    let size = (max - min + IVec3::ONE).as_vec3();
    let half = size * 0.5;
    let edges = bounds_edge_transforms(center, size, half);

    for (part, mut transform, mut visibility) in parts.iter_mut() {
        match *part {
            DeleteBoundsPart::Fill => {
                *transform = Transform::from_translation(center).with_scale(size);
                *visibility = Visibility::Visible;
            }
            DeleteBoundsPart::Edge(i) => {
                let (pos, scale) = edges[i as usize];
                *transform = Transform::from_translation(pos).with_scale(scale);
                *visibility = Visibility::Visible;
            }
        }
    }
}

/// 计算包围盒 12 条边的世界坐标与缩放
fn bounds_edge_transforms(center: Vec3, size: Vec3, half: Vec3) -> [(Vec3, Vec3); 12] {
    let (x0, x1) = (center.x - half.x, center.x + half.x);
    let (y0, y1) = (center.y - half.y, center.y + half.y);
    let (z0, z1) = (center.z - half.z, center.z + half.z);
    let t = BOUNDS_EDGE_T;
    [
        (Vec3::new(center.x, y0, z0), Vec3::new(size.x, t, t)),
        (Vec3::new(center.x, y0, z1), Vec3::new(size.x, t, t)),
        (Vec3::new(center.x, y1, z0), Vec3::new(size.x, t, t)),
        (Vec3::new(center.x, y1, z1), Vec3::new(size.x, t, t)),
        (Vec3::new(x0, center.y, z0), Vec3::new(t, size.y, t)),
        (Vec3::new(x0, center.y, z1), Vec3::new(t, size.y, t)),
        (Vec3::new(x1, center.y, z0), Vec3::new(t, size.y, t)),
        (Vec3::new(x1, center.y, z1), Vec3::new(t, size.y, t)),
        (Vec3::new(x0, y0, center.z), Vec3::new(t, t, size.z)),
        (Vec3::new(x0, y1, center.z), Vec3::new(t, t, size.z)),
        (Vec3::new(x1, y0, center.z), Vec3::new(t, t, size.z)),
        (Vec3::new(x1, y1, center.z), Vec3::new(t, t, size.z)),
    ]
}

/// 计算包围盒 8 个角点世界坐标
fn bounds_corner_positions(center: Vec3, half: Vec3) -> [Vec3; 8] {
    let (x0, x1) = (center.x - half.x, center.x + half.x);
    let (y0, y1) = (center.y - half.y, center.y + half.y);
    let (z0, z1) = (center.z - half.z, center.z + half.z);
    [
        Vec3::new(x0, y0, z0),
        Vec3::new(x0, y0, z1),
        Vec3::new(x0, y1, z0),
        Vec3::new(x0, y1, z1),
        Vec3::new(x1, y0, z0),
        Vec3::new(x1, y0, z1),
        Vec3::new(x1, y1, z0),
        Vec3::new(x1, y1, z1),
    ]
}

/// 生成带完整模型的放置预览方块
pub fn spawn_block_preview(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
) {
    spawn_block_model(
        commands,
        meshes,
        assets,
        world,
        pos,
        data,
        assets.block_preview_material(data.kind),
        Some(EditPreview),
        None,
        None,
        AnimationTiming::edit(),
        false,
        false,
        true,
        None,
        None,
        None,
    );
}
