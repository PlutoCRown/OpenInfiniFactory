use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::game::world::blocks::{
    all_blocks, block_render_assets, BlockKind, BlockRenderSpec, BlockShape, BlockTexture,
    ModelMaterial, ModelMaterialSpec, ModelMesh, ModelMeshSpec, StampColor, BLOCK_SIZE,
};
use crate::game::world::direction::Facing;
use crate::game::world::procedural_textures::block_texture;

#[derive(Resource, Clone)]
pub struct WorldRenderManager {
    pub(crate) block: Handle<Mesh>,
    node: Handle<Mesh>,
    wire_node: Handle<Mesh>,
    pub(crate) goal_top: Handle<Mesh>,
    pub(crate) face_mark: Handle<Mesh>,
    pub(crate) weld_spark: Handle<Mesh>,
    connector_x: Handle<Mesh>,
    connector_y: Handle<Mesh>,
    connector_z: Handle<Mesh>,
    wire_connector_x: Handle<Mesh>,
    wire_connector_y: Handle<Mesh>,
    wire_connector_z: Handle<Mesh>,
    model_meshes: HashMap<ModelMesh, Handle<Mesh>>,
    block_materials: HashMap<BlockKind, Handle<StandardMaterial>>,
    preview_materials: HashMap<BlockKind, Handle<StandardMaterial>>,
    scene_materials: HashMap<BlockKind, Handle<StandardMaterial>>,
    face_mark_materials: HashMap<StampColor, Handle<StandardMaterial>>,
    model_materials: HashMap<ModelMaterial, Handle<StandardMaterial>>,
    pub(crate) wire_connector_material: Handle<StandardMaterial>,
    pub(crate) active_wire_material: Handle<StandardMaterial>,
    pub(crate) goal_top_material: Handle<StandardMaterial>,
    pub(crate) weld_connector_material: Handle<StandardMaterial>,
    delete_preview_material: Handle<StandardMaterial>,
    selection_preview_material: Handle<StandardMaterial>,
    active_factory_debug_material: Handle<StandardMaterial>,
    inactive_factory_debug_material: Handle<StandardMaterial>,
    material_debug_material: Handle<StandardMaterial>,
}

pub enum EditPreviewKind {
    Delete,
    Selection,
}

impl WorldRenderManager {
    pub(crate) fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        images: &mut Assets<Image>,
    ) -> Self {
        let block_textures: HashMap<_, _> = all_blocks()
            .into_iter()
            .filter_map(|kind| {
                kind.definition()
                    .texture()
                    .map(|texture| (kind, images.add(block_texture(texture))))
            })
            .collect();
        let block_materials = all_blocks()
            .into_iter()
            .map(|kind| {
                let texture = block_textures.get(&kind).cloned();
                let material = texture
                    .map(|texture| textured_block_material(kind, texture))
                    .unwrap_or_else(|| block_material(kind));
                (kind, materials.add(material))
            })
            .collect();
        let preview_materials = all_blocks()
            .into_iter()
            .map(|kind| {
                let texture = block_textures.get(&kind).cloned();
                (kind, materials.add(preview_block_material(kind, texture)))
            })
            .collect();
        let scene_block_materials = all_blocks()
            .into_iter()
            .filter(|kind| kind.is_scene())
            .filter_map(|kind| {
                block_textures.get(&kind).map(|texture| {
                    (
                        kind,
                        materials.add(textured_scene_material(kind.material(), texture.clone())),
                    )
                })
            })
            .collect();
        let face_mark_materials = StampColor::ALL
            .into_iter()
            .map(|color| {
                (
                    color,
                    materials.add(StandardMaterial {
                        base_color: color.color().with_alpha(0.82),
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        ..default()
                    }),
                )
            })
            .collect();
        let model_meshes = block_render_assets()
            .flat_map(|assets| assets.meshes.iter().copied())
            .map(|(mesh, spec)| (mesh, meshes.add(model_mesh(spec))))
            .collect();
        let model_materials = block_render_assets()
            .flat_map(|assets| assets.materials.iter().copied())
            .map(|(material, spec)| {
                (
                    material,
                    materials.add(model_material(spec, &block_textures, images)),
                )
            })
            .collect();

        Self {
            block: meshes.add(Cuboid::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE)),
            node: meshes.add(Cuboid::new(
                BLOCK_SIZE * 0.38,
                BLOCK_SIZE * 0.38,
                BLOCK_SIZE * 0.38,
            )),
            wire_node: meshes.add(Cuboid::new(
                BLOCK_SIZE * 0.304,
                BLOCK_SIZE * 0.304,
                BLOCK_SIZE * 0.304,
            )),
            goal_top: meshes.add(Cuboid::new(0.62, 0.08, 0.62)),
            face_mark: meshes.add(Cuboid::new(0.72, 0.012, 0.72)),
            weld_spark: meshes.add(Cuboid::new(0.24, 0.24, 0.24)),
            connector_x: meshes.add(Cuboid::new(0.55, 0.10, 0.10)),
            connector_y: meshes.add(Cuboid::new(0.10, 0.55, 0.10)),
            connector_z: meshes.add(Cuboid::new(0.10, 0.10, 0.55)),
            wire_connector_x: meshes.add(Cuboid::new(0.652, 0.304, 0.304)),
            wire_connector_y: meshes.add(Cuboid::new(0.304, 0.652, 0.304)),
            wire_connector_z: meshes.add(Cuboid::new(0.304, 0.304, 0.652)),
            model_meshes,
            block_materials,
            preview_materials,
            scene_materials: scene_block_materials,
            face_mark_materials,
            model_materials,
            wire_connector_material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.88, 0.30),
                emissive: Color::srgb(0.20, 0.12, 0.02).into(),
                ..default()
            }),
            active_wire_material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.08, 0.04),
                emissive: Color::srgb(0.34, 0.02, 0.01).into(),
                ..default()
            }),
            goal_top_material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.75, 1.0, 0.55),
                emissive: Color::srgb(0.12, 0.28, 0.08).into(),
                ..default()
            }),
            weld_connector_material: materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.22, 0.10, 0.72),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
            delete_preview_material: materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.08, 0.04, 0.38),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
            selection_preview_material: materials.add(StandardMaterial {
                base_color: Color::srgba(0.25, 0.95, 0.88, 0.34),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
            active_factory_debug_material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.12, 0.90, 0.22),
                unlit: true,
                ..default()
            }),
            inactive_factory_debug_material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.95, 0.12, 0.08),
                unlit: true,
                ..default()
            }),
            material_debug_material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.12, 0.50, 1.0),
                unlit: true,
                ..default()
            }),
        }
    }

    pub(crate) fn block_mesh(&self, kind: BlockKind) -> Handle<Mesh> {
        self.block_mesh_for_spec(self.block_render_spec(kind, Facing::North))
    }

    pub(crate) fn block_mesh_for_spec(&self, spec: BlockRenderSpec) -> Handle<Mesh> {
        match spec.definition.shape() {
            BlockShape::Cube => self.block.clone(),
            BlockShape::Node => self.node.clone(),
        }
    }

    pub(crate) fn block_render_spec(&self, kind: BlockKind, facing: Facing) -> BlockRenderSpec {
        kind.render_spec(facing)
    }

    pub(crate) fn wire_node_mesh(&self) -> Handle<Mesh> {
        self.wire_node.clone()
    }

    pub(crate) fn block_material(&self, kind: BlockKind) -> Handle<StandardMaterial> {
        self.block_materials
            .get(&kind)
            .expect("every block kind has a material")
            .clone()
    }

    pub(crate) fn active_factory_debug_material(&self) -> Handle<StandardMaterial> {
        self.active_factory_debug_material.clone()
    }

    pub(crate) fn inactive_factory_debug_material(&self) -> Handle<StandardMaterial> {
        self.inactive_factory_debug_material.clone()
    }

    pub(crate) fn material_debug_material(&self) -> Handle<StandardMaterial> {
        self.material_debug_material.clone()
    }

    pub(crate) fn edit_preview_material(&self, kind: EditPreviewKind) -> Handle<StandardMaterial> {
        match kind {
            EditPreviewKind::Delete => self.delete_preview_material.clone(),
            EditPreviewKind::Selection => self.selection_preview_material.clone(),
        }
    }

    pub(crate) fn block_preview_material(&self, kind: BlockKind) -> Handle<StandardMaterial> {
        self.preview_materials
            .get(&kind)
            .expect("every block kind has a preview material")
            .clone()
    }

    pub(crate) fn scene_material(&self, kind: BlockKind) -> Option<Handle<StandardMaterial>> {
        self.scene_materials.get(&kind).cloned()
    }

    pub(crate) fn connector_mesh(&self, offset: IVec3) -> Handle<Mesh> {
        if offset.x != 0 {
            self.connector_x.clone()
        } else if offset.y != 0 {
            self.connector_y.clone()
        } else {
            self.connector_z.clone()
        }
    }

    pub(crate) fn wire_connector_mesh(&self, offset: IVec3) -> Handle<Mesh> {
        if offset.x != 0 {
            self.wire_connector_x.clone()
        } else if offset.y != 0 {
            self.wire_connector_y.clone()
        } else {
            self.wire_connector_z.clone()
        }
    }

    pub(crate) fn face_mark_material(&self, color: StampColor) -> Handle<StandardMaterial> {
        self.face_mark_materials
            .get(&color)
            .expect("every stamp color has a material")
            .clone()
    }

    pub(crate) fn model_mesh(&self, mesh: ModelMesh) -> Handle<Mesh> {
        self.model_meshes
            .get(&mesh)
            .expect("every model mesh exists")
            .clone()
    }

    pub(crate) fn model_material(&self, material: ModelMaterial) -> Handle<StandardMaterial> {
        self.model_materials
            .get(&material)
            .expect("every model material exists")
            .clone()
    }
}

fn block_material(kind: BlockKind) -> StandardMaterial {
    let mut material = StandardMaterial {
        base_color: kind.material(),
        perceptual_roughness: 0.88,
        reflectance: 0.18,
        ..default()
    };
    if kind.is_transparent() {
        material.alpha_mode = AlphaMode::Blend;
        material.unlit = kind.is_generated_marker();
    }
    material
}

fn textured_block_material(kind: BlockKind, texture: Handle<Image>) -> StandardMaterial {
    StandardMaterial {
        base_color: kind.material(),
        base_color_texture: Some(texture),
        perceptual_roughness: 0.94,
        reflectance: 0.10,
        ..default()
    }
}

fn preview_block_material(kind: BlockKind, texture: Option<Handle<Image>>) -> StandardMaterial {
    StandardMaterial {
        base_color: kind.material().with_alpha(0.46),
        base_color_texture: texture,
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.94,
        reflectance: 0.08,
        ..default()
    }
}

fn textured_scene_material(base_color: Color, texture: Handle<Image>) -> StandardMaterial {
    StandardMaterial {
        base_color,
        base_color_texture: Some(texture),
        perceptual_roughness: 0.96,
        reflectance: 0.08,
        ..default()
    }
}

fn cover_cuboid_mesh(size: Vec3) -> Mesh {
    let min = -size * 0.5;
    let max = size * 0.5;
    let faces = [
        (
            [
                [min.x, min.y, max.z],
                [max.x, min.y, max.z],
                [max.x, max.y, max.z],
                [min.x, max.y, max.z],
            ],
            [0.0, 0.0, 1.0],
            size.x,
            size.y,
        ),
        (
            [
                [min.x, max.y, min.z],
                [max.x, max.y, min.z],
                [max.x, min.y, min.z],
                [min.x, min.y, min.z],
            ],
            [0.0, 0.0, -1.0],
            size.x,
            size.y,
        ),
        (
            [
                [max.x, min.y, min.z],
                [max.x, max.y, min.z],
                [max.x, max.y, max.z],
                [max.x, min.y, max.z],
            ],
            [1.0, 0.0, 0.0],
            size.z,
            size.y,
        ),
        (
            [
                [min.x, min.y, max.z],
                [min.x, max.y, max.z],
                [min.x, max.y, min.z],
                [min.x, min.y, min.z],
            ],
            [-1.0, 0.0, 0.0],
            size.z,
            size.y,
        ),
        (
            [
                [max.x, max.y, min.z],
                [min.x, max.y, min.z],
                [min.x, max.y, max.z],
                [max.x, max.y, max.z],
            ],
            [0.0, 1.0, 0.0],
            size.x,
            size.z,
        ),
        (
            [
                [max.x, min.y, max.z],
                [min.x, min.y, max.z],
                [min.x, min.y, min.z],
                [max.x, min.y, min.z],
            ],
            [0.0, -1.0, 0.0],
            size.x,
            size.z,
        ),
    ];

    let mut positions = Vec::with_capacity(24);
    let mut normals = Vec::with_capacity(24);
    let mut uvs = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    for (face_index, (face_positions, normal, width, height)) in faces.into_iter().enumerate() {
        let base = face_index as u32 * 4;
        positions.extend_from_slice(&face_positions);
        normals.extend_from_slice(&[normal; 4]);
        uvs.extend_from_slice(&cover_uvs(width, height));
        indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

fn cover_uvs(width: f32, height: f32) -> [[f32; 2]; 4] {
    let min_side = width.min(height).max(f32::EPSILON);
    let u_span = width / min_side;
    let v_span = height / min_side;
    let u0 = 0.5 - u_span * 0.5;
    let u1 = 0.5 + u_span * 0.5;
    let v0 = 0.5 - v_span * 0.5;
    let v1 = 0.5 + v_span * 0.5;
    [[u0, v0], [u1, v0], [u1, v1], [u0, v1]]
}

fn model_mesh(spec: ModelMeshSpec) -> Mesh {
    match spec {
        ModelMeshSpec::Cuboid { size } => Cuboid::new(size[0], size[1], size[2]).into(),
        ModelMeshSpec::CoveredCuboid { size } => cover_cuboid_mesh(Vec3::from_array(size)),
        ModelMeshSpec::Cylinder {
            radius,
            height,
            resolution,
        } => Cylinder::new(radius, height)
            .mesh()
            .resolution(resolution)
            .into(),
        ModelMeshSpec::Ring {
            outer_radius,
            inner_radius,
            height,
            segments,
        } => rotator_ring_mesh(outer_radius, inner_radius, height, segments),
        ModelMeshSpec::DrillTip {
            radius,
            length,
            segments,
        } => drill_tip_mesh(radius, length, segments),
    }
}

fn model_material(
    spec: ModelMaterialSpec,
    block_textures: &HashMap<BlockKind, Handle<Image>>,
    images: &mut Assets<Image>,
) -> StandardMaterial {
    match spec {
        ModelMaterialSpec::Srgb { color } => StandardMaterial {
            base_color: color.color(),
            perceptual_roughness: 0.82,
            reflectance: 0.16,
            ..default()
        },
        ModelMaterialSpec::Emissive { color, emissive } => StandardMaterial {
            base_color: color.color(),
            emissive: emissive.color().into(),
            perceptual_roughness: 0.72,
            reflectance: 0.10,
            ..default()
        },
        ModelMaterialSpec::Textured { color, texture } => StandardMaterial {
            base_color: color.color(),
            base_color_texture: Some(model_texture(texture, block_textures, images)),
            perceptual_roughness: 0.90,
            reflectance: 0.12,
            ..default()
        },
    }
}

fn model_texture(
    texture: BlockTexture,
    block_textures: &HashMap<BlockKind, Handle<Image>>,
    images: &mut Assets<Image>,
) -> Handle<Image> {
    block_textures
        .iter()
        .find_map(|(kind, handle)| {
            (kind.definition().texture() == Some(texture)).then(|| handle.clone())
        })
        .unwrap_or_else(|| images.add(block_texture(texture)))
}

fn rotator_ring_mesh(outer_radius: f32, inner_radius: f32, height: f32, segments: u32) -> Mesh {
    let half_height = height * 0.5;
    let mut positions = Vec::with_capacity((segments * 8) as usize);
    let mut normals = Vec::with_capacity((segments * 8) as usize);
    let mut uvs = Vec::with_capacity((segments * 8) as usize);
    let mut indices = Vec::with_capacity((segments * 24) as usize);

    for i in 0..segments {
        let angle = i as f32 / segments as f32 * std::f32::consts::TAU;
        let (sin, cos) = angle.sin_cos();
        let outer = [cos * outer_radius, sin * outer_radius];
        let inner = [cos * inner_radius, sin * inner_radius];

        positions.extend_from_slice(&[
            [outer[0], half_height, outer[1]],
            [inner[0], half_height, inner[1]],
            [outer[0], -half_height, outer[1]],
            [inner[0], -half_height, inner[1]],
            [outer[0], half_height, outer[1]],
            [outer[0], -half_height, outer[1]],
            [inner[0], half_height, inner[1]],
            [inner[0], -half_height, inner[1]],
        ]);
        normals.extend_from_slice(&[
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [cos, 0.0, sin],
            [cos, 0.0, sin],
            [-cos, 0.0, -sin],
            [-cos, 0.0, -sin],
        ]);
        let u = i as f32 / segments as f32;
        uvs.extend_from_slice(&[
            [u, 1.0],
            [u, 0.0],
            [u, 1.0],
            [u, 0.0],
            [u, 1.0],
            [u, 0.0],
            [u, 1.0],
            [u, 0.0],
        ]);
    }

    for i in 0..segments {
        let next = (i + 1) % segments;
        let a = i * 8;
        let b = next * 8;
        indices.extend_from_slice(&[
            a,
            a + 1,
            b,
            a + 1,
            b + 1,
            b,
            a + 2,
            b + 2,
            a + 3,
            a + 3,
            b + 2,
            b + 3,
            a + 4,
            b + 4,
            a + 5,
            a + 5,
            b + 4,
            b + 5,
            a + 6,
            a + 7,
            b + 6,
            a + 7,
            b + 7,
            b + 6,
        ]);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

fn drill_tip_mesh(radius: f32, length: f32, segments: u32) -> Mesh {
    let base_z = 0.0;
    let tip_z = -length;
    let mut positions = Vec::with_capacity((segments * 2 + 2) as usize);
    let mut normals = Vec::with_capacity((segments * 2 + 2) as usize);
    let mut uvs = Vec::with_capacity((segments * 2 + 2) as usize);
    let mut indices = Vec::with_capacity((segments * 6) as usize);

    positions.push([0.0, 0.0, tip_z]);
    normals.push([0.0, 0.0, -1.0]);
    uvs.push([0.5, 0.5]);

    for i in 0..segments {
        let angle = i as f32 / segments as f32 * std::f32::consts::TAU;
        let (sin, cos) = angle.sin_cos();
        let normal = Vec3::new(cos, sin, radius / length).normalize();
        positions.push([cos * radius, sin * radius, base_z]);
        normals.push(normal.to_array());
        uvs.push([i as f32 / segments as f32, 1.0]);
    }

    let base_center = positions.len() as u32;
    positions.push([0.0, 0.0, base_z]);
    normals.push([0.0, 0.0, 1.0]);
    uvs.push([0.5, 0.5]);

    let base_start = positions.len() as u32;
    for i in 0..segments {
        let angle = i as f32 / segments as f32 * std::f32::consts::TAU;
        let (sin, cos) = angle.sin_cos();
        positions.push([cos * radius, sin * radius, base_z]);
        normals.push([0.0, 0.0, 1.0]);
        uvs.push([0.5 + cos * 0.5, 0.5 + sin * 0.5]);
    }

    for i in 0..segments {
        let next = if i + 1 == segments { 0 } else { i + 1 };
        indices.extend_from_slice(&[0, 1 + i, 1 + next]);
        indices.extend_from_slice(&[base_center, base_start + next, base_start + i]);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}
