//! 程序化零件网格（无 GLB 时的回退几何）

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

fn block_corner(v: [u8; 3]) -> Vec3 {
    Vec3::new(v[0] as f32 - 0.5, v[1] as f32 - 0.5, v[2] as f32 - 0.5)
}

/// 吸盘四棱锥：工作面为局部 -Z 的 1×1 面，顶点在格子中心
pub(super) fn suction_cup_pyramid_mesh() -> Mesh {
    let base = [
        block_corner([0, 0, 0]),
        block_corner([1, 0, 0]),
        block_corner([1, 1, 0]),
        block_corner([0, 1, 0]),
    ];
    let apex = Vec3::ZERO;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // 工作面（外法线 -Z）
    push_face(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        &base,
        Vec3::NEG_Z,
        &[[0, 3, 2], [0, 2, 1]],
    );

    // 四个侧面：从底边到顶点
    for i in 0..4 {
        let next = (i + 1) % 4;
        let tri = [base[i], base[next], apex];
        let normal = (tri[1] - tri[0]).cross(tri[2] - tri[0]).normalize_or_zero();
        push_face(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            &tri,
            normal,
            &[[0, 1, 2]],
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

const MIRROR_FACE_THICKNESS: f32 = 0.06;

/// 镜子类厚面片（四顶点格子角）
pub(super) fn thick_quad_mesh(corners: [[u8; 3]; 4]) -> Mesh {
    let vertices = corners.map(block_corner);
    thick_face_mesh(&vertices, &[[0, 1, 2], [0, 2, 3]], MIRROR_FACE_THICKNESS)
}

/// 分光镜六边形：先取过中心的 x+y+z=0 切面，再把 -180° yaw 烘焙进顶点
pub(super) fn thick_splitter_hexagon_mesh() -> Mesh {
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

/// 活塞罩壳立方体（覆盖式 UV）
pub(super) fn cover_cuboid_mesh(size: Vec3) -> Mesh {
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

/// 旋转器圆环网格
pub(super) fn rotator_ring_mesh(outer_radius: f32, inner_radius: f32, height: f32, segments: u32) -> Mesh {
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

/// 钻头锥尖网格
pub(super) fn drill_tip_mesh(radius: f32, length: f32, segments: u32) -> Mesh {
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
