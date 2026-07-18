//! 静态场景方块按 chunk 合并 mesh，降低 Transform / 阴影可见性成本

use std::collections::{HashMap, HashSet};

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;

use super::components::GameplayScene;
use super::scene_mesh::append_scene_cube_faces;
use crate::game::blocks::BlockData;
use crate::game::world::grid::{WorldBlocks, grid_to_world};
use crate::game::world::render_assets::WorldRenderAssets;

const SCENE_CHUNK_SIZE: i32 = 16;

/// 合并后的场景 chunk 绘制实体
#[derive(Component)]
pub struct SceneChunkMesh;

struct ChunkDraw {
    entity: Entity,
    mesh: Handle<Mesh>,
}

/// 场景 chunk 合并网格索引
#[derive(Resource, Default)]
pub struct SceneChunkMeshes {
    by_chunk: HashMap<IVec3, Vec<ChunkDraw>>,
    dirty: HashSet<IVec3>,
}

/// 格子坐标 → chunk 坐标
fn chunk_coord(pos: IVec3) -> IVec3 {
    IVec3::new(
        pos.x.div_euclid(SCENE_CHUNK_SIZE),
        pos.y.div_euclid(SCENE_CHUNK_SIZE),
        pos.z.div_euclid(SCENE_CHUNK_SIZE),
    )
}

/// chunk 原点（世界空间，对应 chunk 角格的整数坐标）
fn chunk_origin(chunk: IVec3) -> Vec3 {
    (chunk * SCENE_CHUNK_SIZE).as_vec3()
}

/// 标记 pos 及其 26 邻格所属 chunk 为脏（AO / 面剔除跨边界）
pub fn mark_scene_chunks_dirty_around(scene_chunks: &mut SceneChunkMeshes, pos: IVec3) {
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                scene_chunks
                    .dirty
                    .insert(chunk_coord(pos + IVec3::new(dx, dy, dz)));
            }
        }
    }
}

/// 按一组改动格标脏
pub fn mark_scene_chunks_dirty_for_positions(
    scene_chunks: &mut SceneChunkMeshes,
    positions: impl IntoIterator<Item = IVec3>,
) {
    for pos in positions {
        mark_scene_chunks_dirty_around(scene_chunks, pos);
    }
}

/// 销毁全部场景 chunk 绘制实体
pub fn clear_scene_chunks(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    scene_chunks: &mut SceneChunkMeshes,
) {
    for draws in scene_chunks.by_chunk.values() {
        for draw in draws {
            commands.entity(draw.entity).despawn();
            meshes.remove(draw.mesh.id());
        }
    }
    scene_chunks.by_chunk.clear();
    scene_chunks.dirty.clear();
}

/// 实体已由别处销毁时，只清索引并释放 mesh 资源
pub fn forget_scene_chunks(meshes: &mut Assets<Mesh>, scene_chunks: &mut SceneChunkMeshes) {
    for draws in scene_chunks.by_chunk.values() {
        for draw in draws {
            meshes.remove(draw.mesh.id());
        }
    }
    scene_chunks.by_chunk.clear();
    scene_chunks.dirty.clear();
}

/// 全量重建所有含场景方块的 chunk
pub fn rebuild_all_scene_chunks(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    scene_chunks: &mut SceneChunkMeshes,
) {
    clear_scene_chunks(commands, meshes, scene_chunks);
    let mut needed = HashSet::new();
    for (pos, data) in &world.blocks {
        if data.kind.is_scene() {
            needed.insert(chunk_coord(*pos));
        }
    }
    for chunk in needed {
        rebuild_one_chunk(commands, meshes, world, assets, scene_chunks, chunk);
    }
}

/// 把脏 chunk 刷成最新合并网格
pub fn flush_dirty_scene_chunks(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    scene_chunks: &mut SceneChunkMeshes,
) {
    let dirty: Vec<IVec3> = scene_chunks.dirty.drain().collect();
    for chunk in dirty {
        rebuild_one_chunk(commands, meshes, world, assets, scene_chunks, chunk);
    }
}

/// 按改动格标脏并立刻重建
pub fn sync_scene_chunks_for_positions(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    scene_chunks: &mut SceneChunkMeshes,
    positions: impl IntoIterator<Item = IVec3>,
) {
    mark_scene_chunks_dirty_for_positions(scene_chunks, positions);
    flush_dirty_scene_chunks(commands, meshes, world, assets, scene_chunks);
}

fn rebuild_one_chunk(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    world: &WorldBlocks,
    assets: &WorldRenderAssets,
    scene_chunks: &mut SceneChunkMeshes,
    chunk: IVec3,
) {
    if let Some(draws) = scene_chunks.by_chunk.remove(&chunk) {
        for draw in draws {
            commands.entity(draw.entity).despawn();
            meshes.remove(draw.mesh.id());
        }
    }

    let origin = chunk_origin(chunk);
    let min = chunk * SCENE_CHUNK_SIZE;
    let max = min + IVec3::splat(SCENE_CHUNK_SIZE);

    // material → 合并缓冲
    let mut buckets: HashMap<AssetId<StandardMaterial>, MeshBucket> = HashMap::new();
    let mut material_handles: HashMap<AssetId<StandardMaterial>, Handle<StandardMaterial>> =
        HashMap::new();

    for x in min.x..max.x {
        for y in min.y..max.y {
            for z in min.z..max.z {
                let pos = IVec3::new(x, y, z);
                let Some(data) = world.blocks.get(&pos).copied() else {
                    continue;
                };
                if !data.kind.is_scene() {
                    continue;
                }
                let material = assets
                    .scene_material(data.kind)
                    .unwrap_or_else(|| assets.block_material(data.kind));
                let id = material.id();
                material_handles
                    .entry(id)
                    .or_insert_with(|| material.clone());
                let bucket = buckets.entry(id).or_default();
                append_scene_block(bucket, meshes, assets, world, pos, data, origin);
            }
        }
    }

    if buckets.is_empty() {
        return;
    }

    let mut draws = Vec::with_capacity(buckets.len());
    for (mat_id, bucket) in buckets {
        if bucket.positions.is_empty() {
            continue;
        }
        let Some(material) = material_handles.remove(&mat_id) else {
            continue;
        };
        let mesh = meshes.add(bucket.into_mesh());
        let entity = commands
            .spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(origin),
                Visibility::default(),
                SceneChunkMesh,
                GameplayScene,
                Pickable::IGNORE,
            ))
            .id();
        draws.push(ChunkDraw { entity, mesh });
    }
    if !draws.is_empty() {
        scene_chunks.by_chunk.insert(chunk, draws);
    }
}

#[derive(Default)]
struct MeshBucket {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl MeshBucket {
    fn into_mesh(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_indices(Indices::U32(self.indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
    }
}

fn append_scene_block(
    bucket: &mut MeshBucket,
    meshes: &Assets<Mesh>,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    pos: IVec3,
    data: BlockData,
    origin: Vec3,
) {
    if let Some(face_uvs) = assets.scene_face_uvs(data.kind) {
        append_scene_cube_faces(
            &mut bucket.positions,
            &mut bucket.normals,
            &mut bucket.uvs,
            &mut bucket.colors,
            &mut bucket.indices,
            pos,
            world,
            assets,
            face_uvs,
            origin,
            true,
        );
        return;
    }

    let template = assets
        .scene_mesh(data.kind)
        .unwrap_or_else(|| assets.block_mesh(data.kind));
    let Some(template_mesh) = meshes.get(template.id()) else {
        return;
    };
    let rotation = if data.kind.is_directional() {
        Quat::from_rotation_y(data.facing.yaw())
    } else {
        Quat::IDENTITY
    };
    let translation = grid_to_world(pos) - origin;
    append_transformed_mesh(bucket, template_mesh, translation, rotation);
}

fn append_transformed_mesh(
    bucket: &mut MeshBucket,
    template: &Mesh,
    translation: Vec3,
    rotation: Quat,
) {
    let Some(VertexAttributeValues::Float32x3(src_pos)) =
        template.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return;
    };
    let normals = match template.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(n)) => n.as_slice(),
        _ => &[],
    };
    let uvs = match template.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(u)) => u.as_slice(),
        _ => &[],
    };
    let src_indices: Vec<u32> = match template.indices() {
        Some(Indices::U32(i)) => i.clone(),
        Some(Indices::U16(i)) => i.iter().map(|&v| v as u32).collect(),
        None => (0..src_pos.len() as u32).collect(),
    };

    let base = bucket.positions.len() as u32;
    for (i, p) in src_pos.iter().enumerate() {
        let local = rotation * Vec3::from(*p) + translation;
        bucket.positions.push(local.to_array());
        let n = normals
            .get(i)
            .copied()
            .map(|n| (rotation * Vec3::from(n)).normalize_or_zero().to_array())
            .unwrap_or([0.0, 1.0, 0.0]);
        bucket.normals.push(n);
        bucket.uvs.push(uvs.get(i).copied().unwrap_or([0.0, 0.0]));
        bucket.colors.push([1.0, 1.0, 1.0, 1.0]);
    }
    for idx in src_indices {
        bucket.indices.push(base + idx);
    }
}
