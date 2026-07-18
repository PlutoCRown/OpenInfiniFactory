use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::game::world::grid::WorldBlocks;
use crate::game::world::render_assets::WorldRenderAssets;

/// 场景方块是否遮挡 AO / 邻面（不透明实心立方体）
pub(super) fn scene_block_occludes(
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    pos: IVec3,
) -> bool {
    world
        .blocks
        .get(&pos)
        .is_some_and(|block| assets.scene_occludes_faces(block.kind))
}

/// 顶点环境光遮蔽强度
fn scene_vertex_ao(side1: bool, side2: bool, corner: bool) -> f32 {
    if side1 && side2 {
        0.55
    } else {
        match side1 as u8 + side2 as u8 + corner as u8 {
            0 => 1.0,
            1 => 0.85,
            2 => 0.7,
            _ => 0.55,
        }
    }
}

/// 六面朝向邻格（与 faces 顺序一致：+Z -Z +X -X +Y -Y）
pub(super) const SCENE_FACE_NEIGHBOR: [IVec3; 6] = [
    IVec3::Z,
    IVec3::NEG_Z,
    IVec3::X,
    IVec3::NEG_X,
    IVec3::Y,
    IVec3::NEG_Y,
];

/// 六面四顶点各自采样的邻接偏移（边1、边2、角）
const SCENE_FACE_AO: [[(IVec3, IVec3, IVec3); 4]; 6] = [
    [
        (IVec3::NEG_X, IVec3::NEG_Y, IVec3::new(-1, -1, 0)),
        (IVec3::X, IVec3::NEG_Y, IVec3::new(1, -1, 0)),
        (IVec3::X, IVec3::Y, IVec3::new(1, 1, 0)),
        (IVec3::NEG_X, IVec3::Y, IVec3::new(-1, 1, 0)),
    ],
    [
        (IVec3::X, IVec3::NEG_Y, IVec3::new(1, -1, 0)),
        (IVec3::NEG_X, IVec3::NEG_Y, IVec3::new(-1, -1, 0)),
        (IVec3::NEG_X, IVec3::Y, IVec3::new(-1, 1, 0)),
        (IVec3::X, IVec3::Y, IVec3::new(1, 1, 0)),
    ],
    [
        (IVec3::Z, IVec3::NEG_Y, IVec3::new(0, -1, 1)),
        (IVec3::NEG_Z, IVec3::NEG_Y, IVec3::new(0, -1, -1)),
        (IVec3::NEG_Z, IVec3::Y, IVec3::new(0, 1, -1)),
        (IVec3::Z, IVec3::Y, IVec3::new(0, 1, 1)),
    ],
    [
        (IVec3::NEG_Z, IVec3::NEG_Y, IVec3::new(0, -1, -1)),
        (IVec3::Z, IVec3::NEG_Y, IVec3::new(0, -1, 1)),
        (IVec3::Z, IVec3::Y, IVec3::new(0, 1, 1)),
        (IVec3::NEG_Z, IVec3::Y, IVec3::new(0, 1, -1)),
    ],
    [
        (IVec3::NEG_X, IVec3::Z, IVec3::new(-1, 0, 1)),
        (IVec3::X, IVec3::Z, IVec3::new(1, 0, 1)),
        (IVec3::X, IVec3::NEG_Z, IVec3::new(1, 0, -1)),
        (IVec3::NEG_X, IVec3::NEG_Z, IVec3::new(-1, 0, -1)),
    ],
    [
        (IVec3::NEG_X, IVec3::NEG_Z, IVec3::new(-1, 0, -1)),
        (IVec3::X, IVec3::NEG_Z, IVec3::new(1, 0, -1)),
        (IVec3::X, IVec3::Z, IVec3::new(1, 0, 1)),
        (IVec3::NEG_X, IVec3::Z, IVec3::new(-1, 0, 1)),
    ],
];

/// 把场景立方体可见面追加进合并缓冲（含 AO；可剔除被遮挡面）
pub(super) fn append_scene_cube_faces(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    pos: IVec3,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    face_uvs: &[[f32; 2]; 24],
    origin: Vec3,
    cull_occluded_faces: bool,
) {
    let min = Vec3::splat(-0.5);
    let max = Vec3::splat(0.5);
    let center = crate::game::world::grid::grid_to_world(pos) - origin;

    // 面顺序与 bake 脚本 / glb 约定一致：+Z -Z +X -X +Y -Y
    let faces = [
        (
            [
                [min.x, min.y, max.z],
                [max.x, min.y, max.z],
                [max.x, max.y, max.z],
                [min.x, max.y, max.z],
            ],
            [0.0, 0.0, 1.0],
        ),
        (
            [
                [max.x, min.y, min.z],
                [min.x, min.y, min.z],
                [min.x, max.y, min.z],
                [max.x, max.y, min.z],
            ],
            [0.0, 0.0, -1.0],
        ),
        (
            [
                [max.x, min.y, max.z],
                [max.x, min.y, min.z],
                [max.x, max.y, min.z],
                [max.x, max.y, max.z],
            ],
            [1.0, 0.0, 0.0],
        ),
        (
            [
                [min.x, min.y, min.z],
                [min.x, min.y, max.z],
                [min.x, max.y, max.z],
                [min.x, max.y, min.z],
            ],
            [-1.0, 0.0, 0.0],
        ),
        (
            [
                [min.x, max.y, max.z],
                [max.x, max.y, max.z],
                [max.x, max.y, min.z],
                [min.x, max.y, min.z],
            ],
            [0.0, 1.0, 0.0],
        ),
        (
            [
                [min.x, min.y, min.z],
                [max.x, min.y, min.z],
                [max.x, min.y, max.z],
                [min.x, min.y, max.z],
            ],
            [0.0, -1.0, 0.0],
        ),
    ];

    for (face_index, (face_positions, normal)) in faces.into_iter().enumerate() {
        if cull_occluded_faces
            && scene_block_occludes(world, assets, pos + SCENE_FACE_NEIGHBOR[face_index])
        {
            continue;
        }
        let base = positions.len() as u32;
        for local in face_positions {
            positions.push([
                local[0] + center.x,
                local[1] + center.y,
                local[2] + center.z,
            ]);
        }
        normals.extend_from_slice(&[normal; 4]);
        uvs.extend_from_slice(&face_uvs[face_index * 4..face_index * 4 + 4]);
        for (side1, side2, corner) in SCENE_FACE_AO[face_index] {
            let ao = scene_vertex_ao(
                scene_block_occludes(world, assets, pos + side1),
                scene_block_occludes(world, assets, pos + side2),
                scene_block_occludes(world, assets, pos + corner),
            );
            colors.push([ao, ao, ao, 1.0]);
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

/// 生成带顶点 AO 的场景方块网格；UV 来自 model.glb 的 24 顶点模板
pub(super) fn scene_block_mesh(
    pos: IVec3,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    face_uvs: &[[f32; 2]; 24],
) -> Mesh {
    let mut positions = Vec::with_capacity(24);
    let mut normals = Vec::with_capacity(24);
    let mut uvs = Vec::with_capacity(24);
    let mut colors = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);
    append_scene_cube_faces(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut colors,
        &mut indices,
        pos,
        world,
        assets,
        face_uvs,
        crate::game::world::grid::grid_to_world(pos),
        false,
    );

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
}
