use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::game::blocks::pusher::texture;
use crate::game::blocks::{
    all_blocks, BlockKind, BlockShape, ModelMaterial, ModelMesh, StampColor, BLOCK_SIZE,
};

#[derive(Resource, Clone)]
pub struct WorldRenderAssets {
    pub(crate) block: Handle<Mesh>,
    node: Handle<Mesh>,
    wire_node: Handle<Mesh>,
    pub(crate) face_mark: Handle<Mesh>,
    pub(crate) weld_spark: Handle<Mesh>,
    connector_x: Handle<Mesh>,
    connector_y: Handle<Mesh>,
    connector_z: Handle<Mesh>,
    wire_connector_x: Handle<Mesh>,
    wire_connector_y: Handle<Mesh>,
    wire_connector_z: Handle<Mesh>,
    part_conveyor_base: Handle<Mesh>,
    part_conveyor_belt: Handle<Mesh>,
    part_drill_body: Handle<Mesh>,
    part_drill_tip: Handle<Mesh>,
    part_large: Handle<Mesh>,
    part_medium: Handle<Mesh>,
    part_small: Handle<Mesh>,
    part_plate: Handle<Mesh>,
    part_rotator_base: Handle<Mesh>,
    part_rotator_disk: Handle<Mesh>,
    part_rotator_ring: Handle<Mesh>,
    part_rod_x: Handle<Mesh>,
    part_rod_y: Handle<Mesh>,
    part_rod_z: Handle<Mesh>,
    part_mirror_face: Handle<Mesh>,
    part_vertical_mirror_face: Handle<Mesh>,
    part_splitter_face: Handle<Mesh>,
    part_pusher_body: Handle<Mesh>,
    part_pusher_head: Handle<Mesh>,
    block_materials: HashMap<BlockKind, Handle<StandardMaterial>>,
    preview_materials: HashMap<BlockKind, Handle<StandardMaterial>>,
    scene_materials: HashMap<BlockKind, Handle<StandardMaterial>>,
    face_mark_materials: HashMap<StampColor, Handle<StandardMaterial>>,
    model_materials: HashMap<ModelMaterial, Handle<StandardMaterial>>,
    model_preview_materials: HashMap<ModelMaterial, Handle<StandardMaterial>>,
    pub(crate) wire_connector_material: Handle<StandardMaterial>,
    pub(crate) active_wire_material: Handle<StandardMaterial>,
    pub(crate) weld_connector_material: Handle<StandardMaterial>,
    pub(crate) laser_beam_material: Handle<StandardMaterial>,
    pub(crate) acceptance_spark_material: Handle<StandardMaterial>,
    delete_preview_material: Handle<StandardMaterial>,
    selection_preview_material: Handle<StandardMaterial>,
    active_factory_debug_material: Handle<StandardMaterial>,
    inactive_factory_debug_material: Handle<StandardMaterial>,
}

pub enum EditPreviewKind {
    Delete,
    Selection,
}

impl WorldRenderAssets {
    pub(crate) fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        images: &mut Assets<Image>,
    ) -> Self {
        let block_textures: HashMap<_, _> = all_blocks()
            .into_iter()
            .filter_map(|kind| kind.block_texture().map(|image| (kind, images.add(image))))
            .collect();
        let platform_texture = block_textures
            .get(&BlockKind::Platform)
            .expect("platform defines a texture")
            .clone();
        let stone_texture = block_textures
            .get(&BlockKind::Stone)
            .expect("stone defines a texture")
            .clone();
        let wood_texture = block_textures
            .get(&BlockKind::Planks)
            .expect("planks define a texture")
            .clone();
        let bordered_wood_texture = images.add(texture::bordered_wood());
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
        let model_materials = [
            (ModelMaterial::ConveyorBase, srgb_material(0.16, 0.18, 0.18)),
            (ModelMaterial::ConveyorBelt, srgb_material(0.02, 0.02, 0.02)),
            (ModelMaterial::DrillTip, srgb_material(0.82, 0.84, 0.82)),
            (ModelMaterial::Frame, srgb_material(0.42, 0.44, 0.44)),
            (ModelMaterial::DarkFrame, srgb_material(0.12, 0.13, 0.15)),
            (ModelMaterial::Belt, srgb_material(0.86, 0.46, 0.14)),
            (
                ModelMaterial::BeltStripe,
                emissive_material(1.0, 0.76, 0.28, 0.18, 0.10, 0.02),
            ),
            (
                ModelMaterial::WeldCore,
                emissive_material(1.0, 0.22, 0.10, 0.22, 0.04, 0.02),
            ),
            (
                ModelMaterial::Welding,
                emissive_material(0.18, 0.58, 1.0, 0.02, 0.12, 0.26),
            ),
            (
                ModelMaterial::Wire,
                emissive_material(1.0, 0.88, 0.30, 0.20, 0.12, 0.02),
            ),
            (
                ModelMaterial::Signal,
                emissive_material(0.12, 0.78, 1.0, 0.02, 0.18, 0.24),
            ),
            (
                ModelMaterial::Power,
                emissive_material(1.0, 0.52, 0.20, 0.22, 0.08, 0.02),
            ),
            (ModelMaterial::Pusher, srgb_material(0.54, 0.56, 0.54)),
            (
                ModelMaterial::Platform,
                textured_model_material(Color::WHITE, platform_texture.clone()),
            ),
            (
                ModelMaterial::PlatformBase,
                block_material(BlockKind::Platform),
            ),
            (ModelMaterial::Wood, srgb_material(0.72, 0.46, 0.22)),
            (
                ModelMaterial::WoodTexture,
                textured_model_material(Color::WHITE, wood_texture),
            ),
            (
                ModelMaterial::BorderedWoodTexture,
                textured_model_material(Color::WHITE, bordered_wood_texture),
            ),
            (
                ModelMaterial::StoneTexture,
                textured_model_material(Color::WHITE, stone_texture),
            ),
            (
                ModelMaterial::Lift,
                emissive_material(0.35, 0.82, 1.0, 0.03, 0.16, 0.22),
            ),
            (
                ModelMaterial::Rotation,
                emissive_material(0.70, 0.36, 1.0, 0.11, 0.04, 0.20),
            ),
            (ModelMaterial::Drill, srgb_material(0.06, 0.07, 0.08)),
            (
                ModelMaterial::Laser,
                emissive_material(1.0, 0.10, 0.22, 0.35, 0.01, 0.04),
            ),
            (
                ModelMaterial::Mirror,
                StandardMaterial {
                    base_color: Color::srgb(0.45, 0.88, 1.0),
                    emissive: LinearRgba::new(0.10, 0.22, 0.30, 1.0),
                    alpha_mode: AlphaMode::Blend,
                    perceptual_roughness: 0.72,
                    reflectance: 0.10,
                    cull_mode: None,
                    ..default()
                },
            ),
            (ModelMaterial::System, srgb_material(0.35, 0.28, 0.48)),
            (
                ModelMaterial::SystemAccent,
                emissive_material(0.72, 0.58, 1.0, 0.12, 0.08, 0.24),
            ),
            (
                ModelMaterial::TeleportIn,
                emissive_material(0.18, 0.62, 1.0, 0.02, 0.10, 0.34),
            ),
            (
                ModelMaterial::TeleportOut,
                emissive_material(1.0, 0.54, 0.18, 0.34, 0.10, 0.02),
            ),
        ]
        .into_iter()
        .map(|(kind, material)| (kind, materials.add(material)))
        .collect::<HashMap<_, _>>();
        let model_preview_materials = model_materials
            .iter()
            .map(|(kind, handle)| {
                let source = materials
                    .get(handle)
                    .expect("model material exists")
                    .clone();
                (*kind, materials.add(preview_model_material(source)))
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
            face_mark: meshes.add(Cuboid::new(0.72, 0.012, 0.72)),
            weld_spark: meshes.add(Cuboid::new(0.24, 0.24, 0.24)),
            connector_x: meshes.add(Cuboid::new(0.55, 0.10, 0.10)),
            connector_y: meshes.add(Cuboid::new(0.10, 0.55, 0.10)),
            connector_z: meshes.add(Cuboid::new(0.10, 0.10, 0.55)),
            wire_connector_x: meshes.add(Cuboid::new(0.652, 0.304, 0.304)),
            wire_connector_y: meshes.add(Cuboid::new(0.304, 0.652, 0.304)),
            wire_connector_z: meshes.add(Cuboid::new(0.304, 0.304, 0.652)),
            part_conveyor_base: meshes.add(Cuboid::new(1.0, 0.90, 1.0)),
            part_conveyor_belt: meshes.add(Cuboid::new(0.90, 0.10, 1.0)),
            part_drill_body: meshes.add(Cuboid::new(1.0, 1.0, 0.80)),
            part_drill_tip: meshes.add(drill_tip_mesh(0.34, 1.0, 48)),
            part_large: meshes.add(Cuboid::new(0.72, 0.22, 0.72)),
            part_medium: meshes.add(Cuboid::new(0.44, 0.20, 0.44)),
            part_small: meshes.add(Cuboid::new(0.22, 0.22, 0.22)),
            part_plate: meshes.add(Cuboid::new(0.78, 0.06, 0.78)),
            part_rotator_base: meshes.add(Cuboid::new(1.0, 0.80, 1.0)),
            part_rotator_disk: meshes.add(Cylinder::new(0.40, 0.20).mesh().resolution(48)),
            part_rotator_ring: meshes.add(rotator_ring_mesh(0.50, 0.40, 0.20, 64)),
            part_rod_x: meshes.add(Cuboid::new(0.72, 0.12, 0.12)),
            part_rod_y: meshes.add(Cuboid::new(0.12, 0.72, 0.12)),
            part_rod_z: meshes.add(Cuboid::new(0.12, 0.12, 0.72)),
            // 镜子面片：000, 101, 111, 010
            part_mirror_face: meshes.add(thick_quad_mesh([
                [0, 0, 0],
                [1, 0, 1],
                [1, 1, 1],
                [0, 1, 0],
            ])),
            // 垂直镜子面片：000, 001, 111, 110
            part_vertical_mirror_face: meshes.add(thick_quad_mesh([
                [0, 0, 0],
                [0, 0, 1],
                [1, 1, 1],
                [1, 1, 0],
            ])),
            // 分光镜：x+y+z=0 六边形再烘焙 -180° yaw
            part_splitter_face: meshes.add(thick_splitter_hexagon_mesh()),
            part_pusher_body: meshes.add(cover_cuboid_mesh(Vec3::new(1.0, 1.0, 0.80))),
            part_pusher_head: meshes.add(cover_cuboid_mesh(Vec3::new(1.0, 1.0, 0.20))),
            block_materials,
            preview_materials,
            scene_materials: scene_block_materials,
            face_mark_materials,
            model_materials,
            model_preview_materials,
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
            weld_connector_material: materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.22, 0.10, 0.72),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
            laser_beam_material: materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.12, 0.26, 0.92),
                emissive: LinearRgba::new(0.55, 0.02, 0.10, 1.0),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
            acceptance_spark_material: materials.add(StandardMaterial {
                base_color: Color::srgba(0.45, 1.0, 0.38, 0.82),
                emissive: Color::srgb(0.10, 0.34, 0.08).into(),
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
                base_color: Color::srgba(0.12, 0.90, 0.22, 0.38),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
            inactive_factory_debug_material: materials.add(StandardMaterial {
                base_color: Color::srgba(0.95, 0.12, 0.08, 0.38),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
        }
    }

    pub(crate) fn block_mesh(&self, kind: BlockKind) -> Handle<Mesh> {
        match kind.shape() {
            BlockShape::Cube => self.block.clone(),
            BlockShape::Node => self.node.clone(),
        }
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
        match mesh {
            ModelMesh::ConveyorBase => self.part_conveyor_base.clone(),
            ModelMesh::ConveyorBelt => self.part_conveyor_belt.clone(),
            ModelMesh::DrillBody => self.part_drill_body.clone(),
            ModelMesh::DrillTip => self.part_drill_tip.clone(),
            ModelMesh::Large => self.part_large.clone(),
            ModelMesh::Medium => self.part_medium.clone(),
            ModelMesh::Small => self.part_small.clone(),
            ModelMesh::Plate => self.part_plate.clone(),
            ModelMesh::RotatorBase => self.part_rotator_base.clone(),
            ModelMesh::RotatorDisk => self.part_rotator_disk.clone(),
            ModelMesh::RotatorRing => self.part_rotator_ring.clone(),
            ModelMesh::RodX => self.part_rod_x.clone(),
            ModelMesh::RodY => self.part_rod_y.clone(),
            ModelMesh::RodZ => self.part_rod_z.clone(),
            ModelMesh::MirrorFace => self.part_mirror_face.clone(),
            ModelMesh::VerticalMirrorFace => self.part_vertical_mirror_face.clone(),
            ModelMesh::SplitterFace => self.part_splitter_face.clone(),
            ModelMesh::PusherBody => self.part_pusher_body.clone(),
            ModelMesh::PusherHead => self.part_pusher_head.clone(),
        }
    }

    pub(crate) fn model_material(&self, material: ModelMaterial) -> Handle<StandardMaterial> {
        self.model_materials
            .get(&material)
            .expect("every model material exists")
            .clone()
    }

    pub(crate) fn model_preview_material(
        &self,
        material: ModelMaterial,
    ) -> Handle<StandardMaterial> {
        self.model_preview_materials
            .get(&material)
            .expect("every model material has a preview material")
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

fn preview_model_material(material: StandardMaterial) -> StandardMaterial {
    StandardMaterial {
        base_color: material.base_color.with_alpha(0.46),
        alpha_mode: AlphaMode::Blend,
        ..material
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

fn srgb_material(r: f32, g: f32, b: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb(r, g, b),
        perceptual_roughness: 0.82,
        reflectance: 0.16,
        ..default()
    }
}

fn textured_model_material(base_color: Color, texture: Handle<Image>) -> StandardMaterial {
    StandardMaterial {
        base_color,
        base_color_texture: Some(texture),
        perceptual_roughness: 0.90,
        reflectance: 0.12,
        ..default()
    }
}

fn block_corner(v: [u8; 3]) -> Vec3 {
    Vec3::new(v[0] as f32 - 0.5, v[1] as f32 - 0.5, v[2] as f32 - 0.5)
}

const MIRROR_FACE_THICKNESS: f32 = 0.06;

fn thick_quad_mesh(corners: [[u8; 3]; 4]) -> Mesh {
    let vertices = corners.map(block_corner);
    thick_face_mesh(&vertices, &[[0, 1, 2], [0, 2, 3]], MIRROR_FACE_THICKNESS)
}

// 分光镜六边形：先取过中心的 x+y+z=0 切面，再把 -180° yaw 烘焙进顶点
fn thick_splitter_hexagon_mesh() -> Mesh {
    let yaw = Quat::from_rotation_y(-std::f32::consts::PI);
    // x+y+z=0 与立方体相交的六个边中点，绕向使法线朝向 (1,1,1)
    let vertices = [
        Vec3::new(0.5, -0.5, 0.0),
        Vec3::new(0.5, 0.0, -0.5),
        Vec3::new(0.0, 0.5, -0.5),
        Vec3::new(-0.5, 0.5, 0.0),
        Vec3::new(-0.5, 0.0, 0.5),
        Vec3::new(0.0, -0.5, 0.5),
    ]
    .map(|vertex| yaw * vertex);
    thick_face_mesh(
        &vertices,
        &[[0, 1, 2], [0, 2, 3], [0, 3, 4], [0, 4, 5]],
        MIRROR_FACE_THICKNESS,
    )
}

fn thick_face_mesh(vertices: &[Vec3], front_triangles: &[[usize; 3]], thickness: f32) -> Mesh {
    let normal = face_normal(vertices, front_triangles[0]);
    let back: Vec<Vec3> = vertices
        .iter()
        .map(|vertex| *vertex - normal * thickness)
        .collect();

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    push_face(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        vertices,
        normal,
        front_triangles,
    );
    let back_triangles: Vec<[usize; 3]> = front_triangles
        .iter()
        .map(|triangle| [triangle[0], triangle[2], triangle[1]])
        .collect();
    push_face(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        &back,
        -normal,
        &back_triangles,
    );

    for index in 0..vertices.len() {
        let next = (index + 1) % vertices.len();
        push_side_quad(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            vertices[index],
            vertices[next],
            back[next],
            back[index],
        );
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

fn face_normal(vertices: &[Vec3], triangle: [usize; 3]) -> Vec3 {
    (vertices[triangle[1]] - vertices[triangle[0]])
        .cross(vertices[triangle[2]] - vertices[triangle[0]])
        .normalize_or_zero()
}

fn push_face(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    vertices: &[Vec3],
    normal: Vec3,
    triangles: &[[usize; 3]],
) {
    let base = positions.len() as u32;
    for vertex in vertices {
        positions.push([vertex.x, vertex.y, vertex.z]);
        normals.push([normal.x, normal.y, normal.z]);
        uvs.push([vertex.x + 0.5, vertex.y + 0.5]);
    }
    for triangle in triangles {
        indices.extend(triangle.iter().map(|index| base + *index as u32));
    }
}

fn push_side_quad(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    front_a: Vec3,
    front_b: Vec3,
    back_b: Vec3,
    back_a: Vec3,
) {
    let side_normal = (front_b - front_a)
        .cross(back_a - front_a)
        .normalize_or_zero();
    let base = positions.len() as u32;
    for vertex in [front_a, front_b, back_b, back_a] {
        positions.push([vertex.x, vertex.y, vertex.z]);
        normals.push([side_normal.x, side_normal.y, side_normal.z]);
        uvs.push([vertex.x + 0.5, vertex.z + 0.5]);
    }
    indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
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

fn emissive_material(r: f32, g: f32, b: f32, er: f32, eg: f32, eb: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb(r, g, b),
        emissive: Color::srgb(er, eg, eb).into(),
        perceptual_roughness: 0.72,
        reflectance: 0.10,
        ..default()
    }
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
