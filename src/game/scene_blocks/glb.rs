//! 同步解析场景方块 model.glb / collision.glb，以及工厂多零件 model.glb

use std::collections::HashMap;
use std::path::Path;

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// 从 model.glb 解出并已写入 Assets 的外观句柄
pub struct SceneGltfHandles {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    /// 六面立方体 24 顶点 UV（供世界 AO 网格复用）；非立方体则为 None
    pub face_uvs: Option<[[f32; 2]; 24]>,
}

/// 工厂 GLB 单个 primitive 零件（可带节点分组名）
pub struct FactoryGltfPart {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    /// 来自 glTF 节点名，如 Body / Stage / Head / PosX
    pub group: Option<String>,
}

/// 同步加载 `model.glb` 并注册到 Assets
pub fn load_scene_glb(
    path: &Path,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) -> Result<SceneGltfHandles, String> {
    let (document, buffers, gltf_images) =
        gltf::import(path).map_err(|e| format!("import {}: {e}", path.display()))?;

    let primitive = document
        .meshes()
        .next()
        .and_then(|mesh| mesh.primitives().next())
        .ok_or_else(|| format!("{}: no mesh primitive", path.display()))?;

    let reader = primitive.reader(|buffer| Some(buffers.get(buffer.index())?.0.as_slice()));

    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .ok_or_else(|| format!("{}: missing POSITION", path.display()))?
        .collect();
    let normals: Vec<[f32; 3]> = reader
        .read_normals()
        .map(|iter| iter.collect())
        .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);
    let uvs: Vec<[f32; 2]> = reader
        .read_tex_coords(0)
        .map(|iter| iter.into_f32().collect())
        .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);
    let indices: Vec<u32> = match reader.read_indices() {
        Some(iter) => iter.into_u32().collect(),
        None => (0..positions.len() as u32).collect(),
    };

    if positions.len() != normals.len() || positions.len() != uvs.len() {
        return Err(format!(
            "{}: attribute length mismatch (pos={} nor={} uv={})",
            path.display(),
            positions.len(),
            normals.len(),
            uvs.len()
        ));
    }

    let face_uvs = if uvs.len() == 24 {
        let mut arr = [[0.0; 2]; 24];
        for (i, uv) in uvs.iter().enumerate() {
            arr[i] = *uv;
        }
        Some(arr)
    } else {
        None
    };

    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    let gltf_material = primitive.material();
    let pbr = gltf_material.pbr_metallic_roughness();
    let factor = pbr.base_color_factor();
    let base_color = Color::srgba(factor[0], factor[1], factor[2], factor[3]);

    let base_color_texture = pbr.base_color_texture().and_then(|info| {
        let texture = info.texture();
        let image = gltf_images.get(texture.source().index())?;
        Some(images.add(bevy_image_from_gltf(image, &texture.sampler())?))
    });

    let alpha_mode = match gltf_material.alpha_mode() {
        gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
        gltf::material::AlphaMode::Mask => {
            AlphaMode::Mask(gltf_material.alpha_cutoff().unwrap_or(0.5))
        }
        gltf::material::AlphaMode::Blend => AlphaMode::Blend,
    };
    let cull_mode = if gltf_material.double_sided() {
        None
    } else {
        Some(bevy::render::render_resource::Face::Back)
    };

    let material = StandardMaterial {
        base_color,
        base_color_texture,
        perceptual_roughness: pbr.roughness_factor(),
        metallic: pbr.metallic_factor(),
        reflectance: 0.08,
        alpha_mode,
        cull_mode,
        ..default()
    };

    Ok(SceneGltfHandles {
        mesh: meshes.add(mesh),
        material: materials.add(material),
        face_uvs,
    })
}

/// 同步加载工厂 `model.glb`：全部 mesh×primitive，按节点名分组
pub fn load_factory_glb(
    path: &Path,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) -> Result<Vec<FactoryGltfPart>, String> {
    let (document, buffers, gltf_images) =
        gltf::import(path).map_err(|e| format!("import {}: {e}", path.display()))?;

    // mesh 索引 → 引用该 mesh 的节点名（取第一个）
    let mut mesh_group: HashMap<usize, String> = HashMap::new();
    for node in document.nodes() {
        if let Some(mesh) = node.mesh() {
            mesh_group
                .entry(mesh.index())
                .or_insert_with(|| node.name().unwrap_or("").to_string());
        }
    }

    let mut material_cache: HashMap<usize, Handle<StandardMaterial>> = HashMap::new();
    let mut parts = Vec::new();

    for mesh in document.meshes() {
        let group = mesh_group.get(&mesh.index()).and_then(|name| {
            if name.is_empty() {
                None
            } else {
                Some(name.clone())
            }
        });
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(buffers.get(buffer.index())?.0.as_slice()));
            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .ok_or_else(|| format!("{}: missing POSITION", path.display()))?
                .collect();
            if positions.is_empty() {
                continue;
            }
            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|iter| iter.collect())
                .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);
            let uvs: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|iter| iter.into_f32().collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);
            let indices: Vec<u32> = match reader.read_indices() {
                Some(iter) => iter.into_u32().collect(),
                None => (0..positions.len() as u32).collect(),
            };
            if positions.len() != normals.len() || positions.len() != uvs.len() {
                return Err(format!(
                    "{}: attribute length mismatch (pos={} nor={} uv={})",
                    path.display(),
                    positions.len(),
                    normals.len(),
                    uvs.len()
                ));
            }

            let bevy_mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            )
            .with_inserted_indices(Indices::U32(indices))
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

            let gltf_material = primitive.material();
            let material_handle = match gltf_material.index() {
                Some(index) => {
                    if let Some(handle) = material_cache.get(&index) {
                        handle.clone()
                    } else {
                        let handle = materials.add(standard_material_from_gltf(
                            &gltf_material,
                            &gltf_images,
                            images,
                        ));
                        material_cache.insert(index, handle.clone());
                        handle
                    }
                }
                None => materials.add(standard_material_from_gltf(
                    &gltf_material,
                    &gltf_images,
                    images,
                )),
            };

            parts.push(FactoryGltfPart {
                mesh: meshes.add(bevy_mesh),
                material: material_handle,
                group: group.clone(),
            });
        }
    }

    if parts.is_empty() {
        return Err(format!("{}: no mesh primitive", path.display()));
    }
    Ok(parts)
}

/// 从 glTF 材质建 StandardMaterial（含贴图 / 双面 / alpha）
fn standard_material_from_gltf(
    gltf_material: &gltf::Material<'_>,
    gltf_images: &[gltf::image::Data],
    images: &mut Assets<Image>,
) -> StandardMaterial {
    let pbr = gltf_material.pbr_metallic_roughness();
    let factor = pbr.base_color_factor();
    let base_color = Color::srgba(factor[0], factor[1], factor[2], factor[3]);
    let base_color_texture = pbr.base_color_texture().and_then(|info| {
        let texture = info.texture();
        let image = gltf_images.get(texture.source().index())?;
        Some(images.add(bevy_image_from_gltf(image, &texture.sampler())?))
    });
    let alpha_mode = match gltf_material.alpha_mode() {
        gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
        gltf::material::AlphaMode::Mask => {
            AlphaMode::Mask(gltf_material.alpha_cutoff().unwrap_or(0.5))
        }
        gltf::material::AlphaMode::Blend => AlphaMode::Blend,
    };
    let cull_mode = if gltf_material.double_sided() {
        None
    } else {
        Some(bevy::render::render_resource::Face::Back)
    };
    let emissive = {
        let e = gltf_material.emissive_factor();
        LinearRgba::new(e[0], e[1], e[2], 1.0)
    };
    StandardMaterial {
        base_color,
        base_color_texture,
        emissive,
        perceptual_roughness: pbr.roughness_factor(),
        metallic: pbr.metallic_factor(),
        alpha_mode,
        cull_mode,
        ..default()
    }
}

/// 从 collision.glb 读出局部空间三角形（与 model 同坐标系，中心在原点）
pub fn load_collision_triangles(path: &Path) -> Result<Vec<[Vec3; 3]>, String> {
    let (document, buffers, _) =
        gltf::import(path).map_err(|e| format!("import {}: {e}", path.display()))?;

    let primitive = document
        .meshes()
        .next()
        .and_then(|mesh| mesh.primitives().next())
        .ok_or_else(|| format!("{}: no mesh primitive", path.display()))?;

    let reader = primitive.reader(|buffer| Some(buffers.get(buffer.index())?.0.as_slice()));
    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .ok_or_else(|| format!("{}: missing POSITION", path.display()))?
        .collect();
    let indices: Vec<u32> = match reader.read_indices() {
        Some(iter) => iter.into_u32().collect(),
        None => (0..positions.len() as u32).collect(),
    };
    if indices.len() % 3 != 0 {
        return Err(format!("{}: index count not multiple of 3", path.display()));
    }

    let mut tris = Vec::with_capacity(indices.len() / 3);
    for tri in indices.chunks_exact(3) {
        let a = positions[tri[0] as usize];
        let b = positions[tri[1] as usize];
        let c = positions[tri[2] as usize];
        tris.push([Vec3::from(a), Vec3::from(b), Vec3::from(c)]);
    }
    Ok(tris)
}

/// 按 glTF sampler 建 Bevy 贴图（NEAREST = 像素锐利，LINEAR = 平滑）
fn bevy_image_from_gltf(
    image: &gltf::image::Data,
    sampler: &gltf::texture::Sampler<'_>,
) -> Option<Image> {
    let (format, pixels) = match image.format {
        gltf::image::Format::R8G8B8A8 => (TextureFormat::Rgba8UnormSrgb, image.pixels.clone()),
        gltf::image::Format::R8G8B8 => {
            let mut rgba = Vec::with_capacity(image.pixels.len() / 3 * 4);
            for chunk in image.pixels.chunks_exact(3) {
                rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
            }
            (TextureFormat::Rgba8UnormSrgb, rgba)
        }
        gltf::image::Format::R8 => (TextureFormat::R8Unorm, image.pixels.clone()),
        gltf::image::Format::R8G8 => (TextureFormat::Rg8Unorm, image.pixels.clone()),
        _ => return None,
    };

    let mut bevy_image = Image::new(
        Extent3d {
            width: image.width,
            height: image.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        format,
        RenderAssetUsages::default(),
    );
    bevy_image.sampler = ImageSampler::Descriptor(sampler_descriptor_from_gltf(sampler));
    Some(bevy_image)
}

/// 把 glTF sampler 的 mag/min/wrap 映射到 Bevy
fn sampler_descriptor_from_gltf(sampler: &gltf::texture::Sampler<'_>) -> ImageSamplerDescriptor {
    use gltf::texture::{MagFilter, MinFilter, WrappingMode};

    let mag_filter = match sampler.mag_filter() {
        Some(MagFilter::Nearest) => ImageFilterMode::Nearest,
        Some(MagFilter::Linear) | None => ImageFilterMode::Linear,
    };
    let (min_filter, mipmap_filter) = match sampler.min_filter() {
        Some(MinFilter::Nearest) => (ImageFilterMode::Nearest, ImageFilterMode::Nearest),
        Some(MinFilter::Linear) => (ImageFilterMode::Linear, ImageFilterMode::Linear),
        Some(MinFilter::NearestMipmapNearest) => {
            (ImageFilterMode::Nearest, ImageFilterMode::Nearest)
        }
        Some(MinFilter::LinearMipmapNearest) => (ImageFilterMode::Linear, ImageFilterMode::Nearest),
        Some(MinFilter::NearestMipmapLinear) => (ImageFilterMode::Nearest, ImageFilterMode::Linear),
        Some(MinFilter::LinearMipmapLinear) | None => {
            (ImageFilterMode::Linear, ImageFilterMode::Linear)
        }
    };
    let address = |mode: WrappingMode| match mode {
        WrappingMode::ClampToEdge => ImageAddressMode::ClampToEdge,
        WrappingMode::MirroredRepeat => ImageAddressMode::MirrorRepeat,
        WrappingMode::Repeat => ImageAddressMode::Repeat,
    };

    ImageSamplerDescriptor {
        mag_filter,
        min_filter,
        mipmap_filter,
        address_mode_u: address(sampler.wrap_s()),
        address_mode_v: address(sampler.wrap_t()),
        address_mode_w: address(sampler.wrap_s()),
        ..default()
    }
}
