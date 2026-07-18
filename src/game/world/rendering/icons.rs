use bevy::camera::visibility::RenderLayers;
use bevy::camera::{RenderTarget, ScalingMode};
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

use super::components::{
    BlockIconAssets, BlockIconRenderCamera, BlockIconRenderEntity, BlockIconRenderRoot,
    BlockIconRenderState,
};
use super::spawn::spawn_block_model;
use crate::game::blocks::{BlockData, BlockKind, PLAY_BLOCKS, edit_blocks};
use crate::game::material_blocks::{
    MaterialBlockRegistry, PaintMaterialRegistry, StampMaterialRegistry,
};
use crate::game::scene_blocks::{SceneBlockRegistry, load_icon_png};
use crate::game::world::animation::AnimationTiming;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::render_assets::WorldRenderAssets;
use crate::game::world::rendering::environment_map_light;
use crate::shared::save::PuzzleLighting;

/// 工厂等仍走离屏渲染的图标尺寸
const ICON_TEXTURE_SIZE: u32 = 256;
const ICON_RENDER_LAYER: usize = 3;
const ICON_SPACING: f32 = 4.0;
const ICON_RENDER_FRAMES: u8 = 3;
/// 与 bake_scene_icons 一致的取景，减少四周留白
const ICON_ORTHO_SIZE: f32 = 1.55;
const ICON_CAMERA_OFFSET: Vec3 = Vec3::new(2.8, 2.2, 2.8);

/// 为 UI 准备方块图标：场景/材料/印花读预烘焙 icon.png，其余离屏渲
pub fn setup_block_icons(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<WorldRenderAssets>,
    scene_registry: Res<SceneBlockRegistry>,
    material_registry: Res<MaterialBlockRegistry>,
    stamp_registry: Res<StampMaterialRegistry>,
    paint_registry: Res<PaintMaterialRegistry>,
    lighting: Res<PuzzleLighting>,
) {
    let icon_layer = RenderLayers::layer(ICON_RENDER_LAYER);
    let mut icon_assets = BlockIconAssets::default();
    let icon_world = WorldBlocks::default();

    // 场景块：只加载 bake 好的 icon.png，不在启动时离屏渲染
    for kind in scene_registry.ordered_kinds() {
        let Some(presentation) = scene_registry.get_kind(kind) else {
            continue;
        };
        let Some(icon_path) = presentation.icon_path.as_ref() else {
            bevy::log::warn!(
                "scene block `{}` missing icon.png (run bake_scene_icons)",
                presentation.string_id
            );
            continue;
        };
        match load_icon_png(icon_path, &mut images) {
            Some(handle) => {
                icon_assets.icons.insert(kind, handle);
            }
            None => {
                bevy::log::warn!("failed to load icon {}", icon_path.display());
            }
        }
    }

    // 材料 / 印花：直接读资源包 icon.png
    for presentation in material_registry.ordered() {
        let kind = BlockKind::Material(presentation.id);
        let Some(icon_path) = presentation.icon_path.as_ref() else {
            bevy::log::warn!("material `{}` missing icon.png", presentation.string_id);
            continue;
        };
        match load_icon_png(icon_path, &mut images) {
            Some(handle) => {
                icon_assets.icons.insert(kind, handle);
            }
            None => {
                bevy::log::warn!("failed to load icon {}", icon_path.display());
            }
        }
    }
    for presentation in stamp_registry.ordered() {
        let kind = BlockKind::Stamp(presentation.id);
        let Some(icon_path) = presentation.icon_path.as_ref() else {
            bevy::log::warn!("stamp `{}` missing icon.png", presentation.string_id);
            continue;
        };
        match load_icon_png(icon_path, &mut images) {
            Some(handle) => {
                icon_assets.icons.insert(kind, handle);
            }
            None => {
                bevy::log::warn!("failed to load icon {}", icon_path.display());
            }
        }
    }

    // 滚刷漆：用 texture.png 作选择格图标
    for presentation in paint_registry.ordered() {
        match load_icon_png(&presentation.texture_path, &mut images) {
            Some(handle) => {
                icon_assets.paints.insert(presentation.id, handle);
            }
            None => {
                bevy::log::warn!(
                    "failed to load paint texture {}",
                    presentation.texture_path.display()
                );
            }
        }
    }

    // 兜底方块：程序化缺失贴图作图标
    {
        use crate::game::blocks::{fallback_material_id, fallback_scene_id};
        let icon = images.add(crate::game::world::procedural_textures::missing_texture_image());
        icon_assets
            .icons
            .insert(BlockKind::Material(fallback_material_id()), icon.clone());
        icon_assets
            .icons
            .insert(BlockKind::Scene(fallback_scene_id()), icon);
    }

    let icon_kinds: Vec<BlockKind> = block_icon_kinds()
        .into_iter()
        .filter(|kind| {
            !kind.is_scene() && !matches!(kind, BlockKind::Material(_) | BlockKind::Stamp(_))
        })
        .collect();
    let selection_glb = {
        use std::path::PathBuf;
        PathBuf::from(crate::shared::platform::asset_path())
            .join("factory_blocks/selection/model.glb")
    };
    let selection_handles = crate::game::scene_blocks::load_scene_glb(
        &selection_glb,
        &mut meshes,
        &mut materials,
        &mut images,
    )
    .map_err(|err| {
        bevy::log::warn!("selection icon glb: {err}");
        err
    })
    .ok();
    let has_offscreen = !icon_kinds.is_empty() || selection_handles.is_some();
    let selection_origin_index = icon_kinds.len();

    if has_offscreen {
        commands.spawn((
            DirectionalLight {
                illuminance: 7800.0,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.85, -0.55, -0.25)),
            icon_layer.clone(),
            BlockIconRenderEntity,
            BlockIconRenderRoot,
        ));
    }

    for (index, kind) in icon_kinds.into_iter().enumerate() {
        let image = Image::new_target_texture(
            ICON_TEXTURE_SIZE,
            ICON_TEXTURE_SIZE,
            TextureFormat::Rgba8Unorm,
            Some(TextureFormat::Rgba8UnormSrgb),
        );
        let image_handle = images.add(image);
        icon_assets.icons.insert(kind, image_handle.clone());

        let origin = Vec3::new(index as f32 * ICON_SPACING, -100.0, 0.0);
        spawn_block_icon_model(
            &mut commands,
            &mut meshes,
            &assets,
            &icon_world,
            kind,
            origin,
            &icon_layer,
        );

        commands.spawn((
            Camera3d::default(),
            Camera {
                order: -2,
                clear_color: Color::NONE.into(),
                ..default()
            },
            RenderTarget::Image(image_handle.into()),
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::Fixed {
                    width: ICON_ORTHO_SIZE,
                    height: ICON_ORTHO_SIZE,
                },
                ..OrthographicProjection::default_3d()
            }),
            Transform::from_translation(origin + ICON_CAMERA_OFFSET).looking_at(origin, Vec3::Y),
            AmbientLight {
                color: Color::WHITE,
                brightness: 520.0,
                ..default()
            },
            environment_map_light(&mut images, &lighting),
            icon_layer.clone(),
            BlockIconRenderEntity,
            BlockIconRenderRoot,
            BlockIconRenderCamera,
        ));
    }

    if let Some(handles) = selection_handles {
        let image = Image::new_target_texture(
            ICON_TEXTURE_SIZE,
            ICON_TEXTURE_SIZE,
            TextureFormat::Rgba8Unorm,
            Some(TextureFormat::Rgba8UnormSrgb),
        );
        let image_handle = images.add(image);
        icon_assets.selection = Some(image_handle.clone());
        let origin = Vec3::new(selection_origin_index as f32 * ICON_SPACING, -100.0, 0.0);
        commands.spawn((
            Mesh3d(handles.mesh),
            MeshMaterial3d(handles.material),
            Transform::from_translation(origin - Vec3::splat(0.5)),
            icon_layer.clone(),
            BlockIconRenderEntity,
            BlockIconRenderRoot,
        ));
        commands.spawn((
            Camera3d::default(),
            Camera {
                order: -2,
                clear_color: Color::NONE.into(),
                ..default()
            },
            RenderTarget::Image(image_handle.into()),
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::Fixed {
                    width: ICON_ORTHO_SIZE,
                    height: ICON_ORTHO_SIZE,
                },
                ..OrthographicProjection::default_3d()
            }),
            Transform::from_translation(origin + ICON_CAMERA_OFFSET).looking_at(origin, Vec3::Y),
            AmbientLight {
                color: Color::WHITE,
                brightness: 520.0,
                ..default()
            },
            environment_map_light(&mut images, &lighting),
            icon_layer.clone(),
            BlockIconRenderEntity,
            BlockIconRenderRoot,
            BlockIconRenderCamera,
        ));
    }

    commands.insert_resource(icon_assets);
    if has_offscreen {
        commands.insert_resource(BlockIconRenderState {
            frames_remaining: ICON_RENDER_FRAMES,
        });
    }
}

/// 需要离屏生成图标的方块种类（不含场景 / 材料 / 印花）
fn block_icon_kinds() -> Vec<BlockKind> {
    let mut kinds = Vec::new();
    for kind in edit_blocks().into_iter().chain(PLAY_BLOCKS) {
        if kind.is_scene() || matches!(kind, BlockKind::Material(_) | BlockKind::Stamp(_)) {
            continue;
        }
        if !kinds.contains(&kind) {
            kinds.push(kind);
        }
    }
    kinds
}

/// 图标拍够帧后关闭相机并销毁离屏实体
pub fn retire_block_icon_renderers(
    mut commands: Commands,
    state: Option<ResMut<BlockIconRenderState>>,
    render_entities: Query<Entity, With<BlockIconRenderRoot>>,
    mut cameras: Query<&mut Camera, With<BlockIconRenderCamera>>,
) {
    let Some(mut state) = state else {
        return;
    };
    if state.frames_remaining > 0 {
        state.frames_remaining -= 1;
        return;
    }

    for mut camera in &mut cameras {
        camera.is_active = false;
    }
    for entity in &render_entities {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<BlockIconRenderState>();
}

/// 在离屏层生成单个方块图标模型
fn spawn_block_icon_model(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &WorldRenderAssets,
    world: &WorldBlocks,
    kind: BlockKind,
    origin: Vec3,
    icon_layer: &RenderLayers,
) {
    let data = BlockData::new(kind, crate::game::world::direction::Facing::South);
    spawn_block_model(
        commands,
        meshes,
        assets,
        world,
        IVec3::ZERO,
        data,
        assets.block_material(data.kind),
        None,
        None,
        None,
        AnimationTiming::edit(),
        false,
        false,
        true,
        Some((origin - Vec3::splat(0.5), icon_layer)),
        None,
        None,
    );
}
