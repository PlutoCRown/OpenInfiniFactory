//! 开发工具：用与游戏相同的离屏相机把场景/材料方块 bake 成 icon.png

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use bevy::app::AppExit;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{RenderTarget, ScalingMode};
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::ExitCondition;

use super::{SceneBlockRegistry, load_global_scene_blocks};
use crate::game::blocks::{BlockData, BlockKind};
use crate::game::material_blocks::{
    MaterialBlockRegistry, MaterialPackRegistries, PaintMaterialRegistry, StampMaterialRegistry,
    load_global_material_packs,
};
use crate::game::world::animation::AnimationTiming;
use crate::game::world::grid::WorldBlocks;
use crate::game::world::render_assets::WorldRenderAssets;
use crate::game::world::rendering::spawn::spawn_block_model;
use crate::game::world::rendering::{BlockIconRenderEntity, BlockIconRenderRoot};
use crate::shared::platform;

const ICON_RENDER_LAYER: usize = 3;
const ICON_SPACING: f32 = 4.0;
/// 正交取景边长；越小方块越满，留白越少（单位立方体斜视约需 ≥1.5）
const ICON_ORTHO_SIZE: f32 = 1.55;
const ICON_CAMERA_OFFSET: Vec3 = Vec3::new(2.8, 2.2, 2.8);
const FRAMES_BEFORE_CAPTURE: u8 = 4;

/// bake 命令行配置
#[derive(Clone, Debug, Resource)]
pub struct BakeSceneIconsConfig {
    /// 输出边长（像素），默认 128
    pub size: u32,
    /// 输出文件名，默认 `icon.png`；做 LOD 时可传 `icon_64.png` 等
    pub output: String,
    /// 只 bake 指定 id；空则全部（场景 + 材料 + 印花）
    pub only: Option<String>,
    /// 场景方块根目录（兼容旧参数；加载仍走全局 assets）
    pub root: PathBuf,
    /// 是否 bake 场景方块
    pub bake_scene: bool,
    /// 是否 bake 材料方块
    pub bake_materials: bool,
    /// 是否 bake 印花材料
    pub bake_stamps: bool,
}

impl Default for BakeSceneIconsConfig {
    fn default() -> Self {
        Self {
            size: 128,
            output: "icon.png".into(),
            only: None,
            root: PathBuf::from(platform::asset_path()).join("scene_blocks"),
            bake_scene: true,
            bake_materials: true,
            bake_stamps: true,
        }
    }
}

/// 解析 argv 并跑 bake（供 `bake_scene_icons` bin 调用）
pub fn run_from_args(args: &[String]) {
    let config = parse_args(args).unwrap_or_else(|err| {
        eprintln!("{err}");
        print_usage();
        std::process::exit(2);
    });
    run(config);
}

fn print_usage() {
    eprintln!(
        "Usage: bake_scene_icons [--size N] [--output NAME] [--only ID]\n\
         \n\
         [--scene-only] [--materials-only] [--stamps-only]\n\
         Defaults: --size 128 --output icon.png（默认同时 bake 场景、材料与印花）\n\
         Example: bake_scene_icons --only iron\n\
         Example: bake_scene_icons --stamps-only\n\
         Example (LOD): bake_scene_icons --size 64 --output icon_64.png"
    );
}

fn parse_args(args: &[String]) -> Result<BakeSceneIconsConfig, String> {
    let mut config = BakeSceneIconsConfig::default();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            "--size" | "-s" => {
                i += 1;
                let v = args
                    .get(i)
                    .ok_or_else(|| "missing value for --size".to_string())?;
                config.size = v.parse().map_err(|_| format!("invalid --size `{v}`"))?;
                if config.size == 0 {
                    return Err("--size must be > 0".into());
                }
            }
            "--output" | "-o" => {
                i += 1;
                config.output = args
                    .get(i)
                    .ok_or_else(|| "missing value for --output".to_string())?
                    .clone();
            }
            "--only" => {
                i += 1;
                config.only = Some(
                    args.get(i)
                        .ok_or_else(|| "missing value for --only".to_string())?
                        .clone(),
                );
            }
            "--root" => {
                i += 1;
                config.root = PathBuf::from(
                    args.get(i)
                        .ok_or_else(|| "missing value for --root".to_string())?,
                );
            }
            "--scene-only" => {
                config.bake_scene = true;
                config.bake_materials = false;
                config.bake_stamps = false;
            }
            "--materials-only" => {
                config.bake_scene = false;
                config.bake_materials = true;
                config.bake_stamps = false;
            }
            "--stamps-only" => {
                config.bake_scene = false;
                config.bake_materials = false;
                config.bake_stamps = true;
            }
            other if other.starts_with("--size=") => {
                let v = &other["--size=".len()..];
                config.size = v.parse().map_err(|_| format!("invalid --size `{v}`"))?;
            }
            other if other.starts_with("--output=") => {
                config.output = other["--output=".len()..].to_string();
            }
            other if other.starts_with("--only=") => {
                config.only = Some(other["--only=".len()..].to_string());
            }
            other if other.starts_with("--root=") => {
                config.root = PathBuf::from(&other["--root=".len()..]);
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
        i += 1;
    }
    Ok(config)
}

/// 启动无头 Bevy，把场景/材料/印花方块渲成 PNG 后退出
pub fn run(config: BakeSceneIconsConfig) {
    let size = config.size;
    App::new()
        .insert_resource(config)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "bake_scene_icons".into(),
                        resolution: (size, size).into(),
                        visible: false,
                        ..default()
                    }),
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: platform::asset_path().into(),
                    ..default()
                }),
        )
        .add_systems(Startup, setup_bake)
        .add_systems(Update, (tick_bake_capture, exit_when_bake_done))
        .run();
}

#[derive(Resource)]
struct BakeRuntime {
    frames_remaining: u8,
    capturing: bool,
    targets: Vec<(Handle<Image>, PathBuf)>,
    remaining_saves: Arc<AtomicUsize>,
}

fn setup_bake(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<BakeSceneIconsConfig>,
) {
    let mut scene_registry = SceneBlockRegistry::default();
    if config.bake_scene {
        if let Err(err) = load_global_scene_blocks(&mut scene_registry) {
            eprintln!("failed to load scene blocks: {err}");
            std::process::exit(1);
        }
    }

    let mut material_registry = MaterialBlockRegistry::default();
    let mut stamp_registry = StampMaterialRegistry::default();
    let mut paint_registry = PaintMaterialRegistry::default();
    if config.bake_materials || config.bake_stamps {
        if let Err(err) = load_global_material_packs(MaterialPackRegistries {
            materials: &mut material_registry,
            stamps: &mut stamp_registry,
            paints: &mut paint_registry,
        }) {
            eprintln!("failed to load material packs: {err}");
            std::process::exit(1);
        }
    }

    commands.insert_resource(scene_registry.clone());
    commands.insert_resource(material_registry.clone());

    let assets = WorldRenderAssets::new(
        &mut meshes,
        &mut materials,
        &mut images,
        &scene_registry,
        &material_registry,
        &stamp_registry,
        &paint_registry,
    );
    let icon_layer = RenderLayers::layer(ICON_RENDER_LAYER);
    let icon_world = WorldBlocks::default();

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

    let mut targets = Vec::new();
    let mut index = 0usize;

    if config.bake_scene {
        for kind in scene_registry.ordered_kinds() {
            let Some(presentation) = scene_registry.get_kind(kind) else {
                continue;
            };
            if let Some(only) = &config.only {
                if presentation.string_id != *only {
                    continue;
                }
            }
            let pack_dir = presentation
                .model_path
                .as_ref()
                .or(presentation.texture_path.as_ref())
                .and_then(|p| p.parent())
                .unwrap_or_else(|| Path::new("."));
            push_bake_target(
                &mut commands,
                &mut images,
                &mut meshes,
                &assets,
                &icon_world,
                &icon_layer,
                &mut targets,
                &mut index,
                kind,
                pack_dir,
                &config.output,
                config.size,
            );
        }
    }

    if config.bake_materials {
        for presentation in material_registry.ordered() {
            if let Some(only) = &config.only {
                if presentation.string_id != *only {
                    continue;
                }
            }
            let pack_dir = presentation
                .model_path
                .as_ref()
                .or(presentation.texture_path.as_ref())
                .or(presentation.icon_path.as_ref())
                .and_then(|p| p.parent())
                .unwrap_or_else(|| Path::new("."));
            push_bake_target(
                &mut commands,
                &mut images,
                &mut meshes,
                &assets,
                &icon_world,
                &icon_layer,
                &mut targets,
                &mut index,
                BlockKind::Material(presentation.id),
                pack_dir,
                &config.output,
                config.size,
            );
        }
    }

    if config.bake_stamps {
        for presentation in stamp_registry.ordered() {
            if let Some(only) = &config.only {
                if presentation.string_id != *only {
                    continue;
                }
            }
            let pack_dir = presentation
                .model_path
                .as_ref()
                .or(presentation.texture_path.as_ref())
                .or(presentation.icon_path.as_ref())
                .and_then(|p| p.parent())
                .unwrap_or_else(|| Path::new("."));
            push_bake_target(
                &mut commands,
                &mut images,
                &mut meshes,
                &assets,
                &icon_world,
                &icon_layer,
                &mut targets,
                &mut index,
                BlockKind::Stamp(presentation.id),
                pack_dir,
                &config.output,
                config.size,
            );
        }
    }

    if targets.is_empty() {
        eprintln!(
            "no blocks to bake (check --only / --scene-only / --materials-only / --stamps-only)"
        );
        std::process::exit(1);
    }

    println!(
        "baking {} icon(s) at {}x{} → {}",
        targets.len(),
        config.size,
        config.size,
        config.output
    );

    commands.insert_resource(assets);
    commands.insert_resource(BakeRuntime {
        frames_remaining: FRAMES_BEFORE_CAPTURE,
        capturing: false,
        remaining_saves: Arc::new(AtomicUsize::new(0)),
        targets,
    });
}

/// 登记一个 bake 目标：离屏纹理 + 相机 + 方块模型
fn push_bake_target(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    meshes: &mut Assets<Mesh>,
    assets: &WorldRenderAssets,
    icon_world: &WorldBlocks,
    icon_layer: &RenderLayers,
    targets: &mut Vec<(Handle<Image>, PathBuf)>,
    index: &mut usize,
    kind: BlockKind,
    pack_dir: &Path,
    output: &str,
    size: u32,
) {
    let out_path = pack_dir.join(output);
    let image = Image::new_target_texture(
        size,
        size,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    );
    let image_handle = images.add(image);
    targets.push((image_handle.clone(), out_path));

    let origin = Vec3::new(*index as f32 * ICON_SPACING, -100.0, 0.0);
    spawn_bake_icon_model(
        commands, meshes, assets, icon_world, kind, origin, icon_layer,
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
        icon_layer.clone(),
        BlockIconRenderEntity,
        BlockIconRenderRoot,
    ));
    *index += 1;
}

fn spawn_bake_icon_model(
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

fn tick_bake_capture(mut commands: Commands, mut runtime: ResMut<BakeRuntime>) {
    if runtime.capturing {
        return;
    }
    if runtime.frames_remaining > 0 {
        runtime.frames_remaining -= 1;
        return;
    }

    runtime.capturing = true;
    let count = runtime.targets.len();
    runtime.remaining_saves.store(count, Ordering::SeqCst);
    let remaining = runtime.remaining_saves.clone();

    for (handle, path) in runtime.targets.clone() {
        let remaining = remaining.clone();
        // 保留 alpha（UI 图标需要透明底）；Bevy 自带 save_to_disk 会丢 alpha 转 RGB
        commands.spawn(Screenshot::image(handle)).observe(
            move |captured: On<ScreenshotCaptured>| {
                save_icon_rgba(&path, &captured.image);
                remaining.fetch_sub(1, Ordering::SeqCst);
            },
        );
    }
}

fn save_icon_rgba(path: &Path, image: &Image) {
    match image.clone().try_into_dynamic() {
        Ok(dyn_img) => {
            let rgba = dyn_img.to_rgba8();
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match rgba.save(path) {
                Ok(()) => println!("wrote {}", path.display()),
                Err(err) => eprintln!("failed to write {}: {err}", path.display()),
            }
        }
        Err(err) => eprintln!(
            "failed to convert screenshot for {}: {err:?}",
            path.display()
        ),
    }
}

fn exit_when_bake_done(runtime: Res<BakeRuntime>, mut exit: MessageWriter<AppExit>) {
    if !runtime.capturing {
        return;
    }
    if runtime.remaining_saves.load(Ordering::SeqCst) == 0 {
        exit.write(AppExit::Success);
    }
}
