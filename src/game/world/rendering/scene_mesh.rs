use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::game::world::grid::WorldBlocks;

/// 场景方块是否遮挡 AO（有碰撞的实体方块）
fn scene_block_occludes(world: &WorldBlocks, pos: IVec3) -> bool {
    world
        .blocks
        .get(&pos)
        .is_some_and(|block| block.kind.has_collision())
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

/// 生成带顶点 AO 的场景方块网格；UV 来自 model.glb 的 24 顶点模板
pub(super) fn scene_block_mesh(pos: IVec3, world: &WorldBlocks, face_uvs: &[[f32; 2]; 24]) -> Mesh {
    let min = Vec3::splat(-0.5);
    let max = Vec3::splat(0.5);

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

    let mut positions = Vec::with_capacity(24);
    let mut normals = Vec::with_capacity(24);
    let mut uvs = Vec::with_capacity(24);
    let mut colors = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);
    for (face_index, (face_positions, normal)) in faces.into_iter().enumerate() {
        let base = (face_index * 4) as u32;
        positions.extend_from_slice(&face_positions);
        normals.extend_from_slice(&[normal; 4]);
        uvs.extend_from_slice(&face_uvs[face_index * 4..face_index * 4 + 4]);
        for (side1, side2, corner) in SCENE_FACE_AO[face_index] {
            let ao = scene_vertex_ao(
                scene_block_occludes(world, pos + side1),
                scene_block_occludes(world, pos + side2),
                scene_block_occludes(world, pos + corner),
            );
            colors.push([ao, ao, ao, 1.0]);
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

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
