//! 启动界面的独立工厂橱窗：真实模拟、等轴渲染，并与玩家会话隔离。

use std::collections::{HashMap, HashSet};

use bevy::camera::{Hdr, RenderTarget, ScalingMode};
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::ecs::system::SystemParam;
use bevy::light::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use bevy::ui::widget::NodeImageMode;
use bevy::window::PrimaryWindow;
use serde::Deserialize;

use crate::game::cameras::{MENU_CLEAR, WorldViewCamera, new_gameplay_view_image};
use crate::game::schedule::GameSet;
use crate::game::session::SessionBusy;
use crate::game::simulation::markers::refresh_static_generated_markers;
use crate::game::simulation::stats::SimulationStepStats;
use crate::game::state::GameMode;
use crate::game::ui::components::default_font_size;
use crate::game::ui::core::StartMenuPage;
use crate::game::ui::core::host::UiRootEntity;
use crate::game::ui::{LocalizedText, UiNavigation};
use crate::game::world::animation::{
    AnimationTiming, BreakDebrisParticle, LaserBeamBurst, SIMULATION_TURN_SECONDS,
    WeldBurstParticle, WeldSpark,
};
use crate::game::world::rendering::{
    BlockEntity, GoalGhostMaterial, PortalFlashQueue, PortalMaterial, SceneChunkMeshes,
    ShadowProxyMaterial, WorldRenderAssets, create_world_render_assets, despawn_world,
    rebuild_world_with_runtime_animations,
};
use crate::scene::{BlockEntityIndex, SceneRenderMut, apply_turn_output};
use crate::shared::config::GameConfig;
use crate::shared::i18n::I18n;
use crate::shared::save::world_from_free_blocks;
use crate::shared::save_format::decode_blocks;
use crate::sim_bridge::SimSession;

/// 橱窗场景切换的淡入淡出时长。
const SHOWCASE_FADE_SECONDS: f32 = 0.35;
/// 橱窗离屏画面的最大宽度，避免高分屏菜单浪费 GPU。
const SHOWCASE_MAX_VIEW_WIDTH: u32 = 1920;
/// 橱窗离屏画面的最大高度。
const SHOWCASE_MAX_VIEW_HEIGHT: u32 = 1080;
/// 启动菜单工厂橱窗插件。
pub struct StartMenuShowcasePlugin;

/// 橱窗淡入、播放和淡出阶段。
#[derive(Clone, Copy, Eq, PartialEq)]
enum ShowcasePhase {
    FadeIn,
    Running,
    Completed,
    FadeOut,
}

/// 启动菜单循环播放的简单工厂场景。
#[derive(Clone, Copy, Eq, PartialEq)]
enum ShowcaseScene {
    Transport,
    Conversion,
    Welding,
    Pusher,
    Lifter,
}

impl ShowcaseScene {
    /// 返回场景稳定 ID，供调试状态和资源目录共同使用。
    fn id(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::Conversion => "conversion",
            Self::Welding => "welding",
            Self::Pusher => "pusher",
            Self::Lifter => "lifter",
        }
    }

    /// 返回场景对应的本地化标题和说明键。
    fn text_keys(self) -> (&'static str, &'static str) {
        match self {
            Self::Transport => ("showcase.transport.title", "showcase.transport.subtitle"),
            Self::Conversion => ("showcase.conversion.title", "showcase.conversion.subtitle"),
            Self::Welding => ("showcase.welding.title", "showcase.welding.subtitle"),
            Self::Pusher => ("showcase.pusher.title", "showcase.pusher.subtitle"),
            Self::Lifter => ("showcase.lifter.title", "showcase.lifter.subtitle"),
        }
    }
}

/// meta.json 中控制单条橱窗播放节奏和等轴构图的配置。
#[derive(Clone, Deserialize)]
struct ShowcasePlayback {
    turn_seconds: f32,
    max_turns: u64,
    completion_hold_seconds: f32,
    camera_position: [f32; 3],
    camera_target: [f32; 3],
    viewport_height: f32,
}

/// 从标准 Free 存档 meta.json 中提取橱窗专用配置。
#[derive(Deserialize)]
struct ShowcaseMetaFile {
    showcase: ShowcasePlayback,
}

/// 与玩家资源分离的橱窗模拟、渲染索引和循环状态。
#[derive(Resource)]
pub(crate) struct StartMenuShowcase {
    session: SimSession,
    scene: ShowcaseScene,
    playback: ShowcasePlayback,
    block_index: BlockEntityIndex,
    scene_chunks: SceneChunkMeshes,
    pub(crate) view: Handle<Image>,
    previous_powered_wires: HashSet<IVec3>,
    turn_elapsed: f32,
    phase_elapsed: f32,
    phase: ShowcasePhase,
}

/// 橱窗专用 3D 相机。
#[derive(Component)]
struct StartMenuShowcaseCamera;

/// 橱窗创建的非方块实体，退出菜单时统一销毁。
#[derive(Component)]
struct StartMenuShowcaseEntity;

/// 菜单 UI 中承载橱窗画面和说明的背景根。
#[derive(Component)]
struct StartMenuShowcaseBackdrop;

/// 需要随循环阶段淡入淡出的橱窗图片节点。
#[derive(Component)]
struct StartMenuShowcaseImage;

/// 橱窗说明文字的标题与副标题位置。
#[derive(Component, Clone, Copy)]
enum StartMenuShowcaseCaption {
    Title,
    Subtitle,
}

/// 橱窗创建世界外观时需要的当前资源包与渲染资产集合。
#[derive(SystemParam)]
struct ShowcaseRenderAssetDeps<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    shadow_proxy_materials: ResMut<'w, Assets<ShadowProxyMaterial>>,
    ghost_materials: ResMut<'w, Assets<GoalGhostMaterial>>,
    portal_materials: ResMut<'w, Assets<PortalMaterial>>,
    images: ResMut<'w, Assets<Image>>,
    scene_registry: Res<'w, crate::game::scene_blocks::SceneBlockRegistry>,
    material_registry: Res<'w, crate::game::material_blocks::MaterialBlockRegistry>,
    stamp_registry: Res<'w, crate::game::material_blocks::StampMaterialRegistry>,
    paint_registry: Res<'w, crate::game::material_blocks::PaintMaterialRegistry>,
}

/// 供嵌入式调试器读取的橱窗模拟、相机、纹理和 UI 快照。
#[derive(SystemParam)]
pub(crate) struct StartMenuShowcaseDebugSnapshot<'w, 's> {
    pub(crate) showcase: Option<Res<'w, StartMenuShowcase>>,
    render_assets: Option<Res<'w, WorldRenderAssets>>,
    images: Res<'w, Assets<Image>>,
    cameras: Query<
        'w,
        's,
        (
            Entity,
            &'static Camera,
            &'static RenderTarget,
            &'static Projection,
            &'static Transform,
        ),
        (
            With<StartMenuShowcaseCamera>,
            Without<crate::game::player::controller::FlyCamera>,
        ),
    >,
    backdrops: Query<'w, 's, &'static Visibility, With<StartMenuShowcaseBackdrop>>,
    image_nodes:
        Query<'w, 's, (&'static ImageNode, &'static ComputedNode), With<StartMenuShowcaseImage>>,
    showcase_entities: Query<'w, 's, Entity, With<StartMenuShowcaseEntity>>,
    block_entities: Query<'w, 's, Entity, (With<StartMenuShowcaseEntity>, With<BlockEntity>)>,
}

impl StartMenuShowcaseDebugSnapshot<'_, '_> {
    /// 生成 `/status.showcase` 诊断对象，资源未建立时返回 unavailable。
    pub(crate) fn capture(&self) -> serde_json::Value {
        let Some(showcase) = self.showcase.as_ref() else {
            return serde_json::json!({
                "available": false,
                "render_assets_ready": self.render_assets.is_some(),
                "camera_count": self.cameras.iter().len(),
                "backdrop_count": self.backdrops.iter().len(),
            });
        };

        let camera = self.cameras.single().ok().map(
            |(entity, camera, render_target, projection, transform)| {
                let (
                    projection_kind,
                    orthographic_scale,
                    orthographic_scaling_mode,
                    orthographic_view_width,
                    orthographic_view_height,
                ) = match projection {
                    Projection::Orthographic(projection) => (
                        "orthographic",
                        Some(projection.scale),
                        Some(format!("{:?}", projection.scaling_mode)),
                        Some(projection.area.max.x - projection.area.min.x),
                        Some(projection.area.max.y - projection.area.min.y),
                    ),
                    Projection::Perspective(_) => ("perspective", None, None, None, None),
                    Projection::Custom(_) => ("custom", None, None, None, None),
                };
                serde_json::json!({
                    "entity": entity.to_bits(),
                    "active": camera.is_active,
                    "order": camera.order,
                    "target_is_image": matches!(render_target, RenderTarget::Image(_)),
                    "projection": projection_kind,
                    "orthographic_scale": orthographic_scale,
                    "orthographic_scaling_mode": orthographic_scaling_mode,
                    "orthographic_view_width": orthographic_view_width,
                    "orthographic_view_height": orthographic_view_height,
                    "position": {
                        "x": transform.translation.x,
                        "y": transform.translation.y,
                        "z": transform.translation.z,
                    },
                })
            },
        );
        let backdrop_visibility = self
            .backdrops
            .single()
            .ok()
            .map(|visibility| format!("{visibility:?}"));
        let image_node = self.image_nodes.single().ok().map(|(node, computed)| {
            serde_json::json!({
                "texture_matches": node.image.id() == showcase.view.id(),
                "alpha": node.color.alpha(),
                "computed_width": computed.size().x,
                "computed_height": computed.size().y,
            })
        });
        let texture = self.images.get(&showcase.view).map(|image| {
            serde_json::json!({
                "width": image.width(),
                "height": image.height(),
            })
        });

        serde_json::json!({
            "available": true,
            "scene": showcase.scene.id(),
            "phase": match showcase.phase {
                ShowcasePhase::FadeIn => "fade_in",
                ShowcasePhase::Running => "running",
                ShowcasePhase::Completed => "completed",
                ShowcasePhase::FadeOut => "fade_out",
            },
            "turn": showcase.session.control().turn,
            "turn_elapsed": showcase.turn_elapsed,
            "turn_seconds": showcase.playback.turn_seconds,
            "max_turns": showcase.playback.max_turns,
            "completion_hold_seconds": showcase.playback.completion_hold_seconds,
            "phase_elapsed": showcase.phase_elapsed,
            "simulation_running": showcase.session.control().running,
            "world": {
                "blocks": showcase.session.world.blocks().len(),
                "system_blocks": showcase.session.world.system_blocks().len(),
                "materials": showcase
                    .session
                    .world
                    .blocks()
                    .values()
                    .filter(|block| block.kind.is_material())
                    .count(),
            },
            "render_assets_ready": self.render_assets.is_some(),
            "camera_count": self.cameras.iter().len(),
            "camera": camera,
            "texture": texture,
            "backdrop_count": self.backdrops.iter().len(),
            "backdrop_visibility": backdrop_visibility,
            "image_node": image_node,
            "tagged_entities": self.showcase_entities.iter().len(),
            "rendered_block_entities": self.block_entities.iter().len(),
        })
    }
}

impl Plugin for StartMenuShowcasePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameMode::StartMenu), teardown_start_menu_showcase)
            .add_systems(
                Update,
                (
                    setup_start_menu_showcase.run_if(not(resource_exists::<StartMenuShowcase>)),
                    sync_start_menu_showcase_visibility,
                    resize_start_menu_showcase_view,
                    advance_start_menu_showcase,
                    mark_start_menu_showcase_entities,
                )
                    .chain()
                    .in_set(GameSet::Menus)
                    .run_if(in_state(GameMode::StartMenu)),
            );
    }
}

/// 按窗口物理比例计算有上限的橱窗离屏尺寸。
fn showcase_view_size(window: &Window) -> (u32, u32) {
    let physical_width = window.physical_width().max(1);
    let physical_height = window.physical_height().max(1);
    let scale = (SHOWCASE_MAX_VIEW_WIDTH as f32 / physical_width as f32)
        .min(SHOWCASE_MAX_VIEW_HEIGHT as f32 / physical_height as f32)
        .min(1.0);
    (
        (physical_width as f32 * scale).round().max(1.0) as u32,
        (physical_height as f32 * scale).round().max(1.0) as u32,
    )
}

/// 从随程序打包的标准 Free 存档构造独立橱窗模拟会话。
fn showcase_session(scene: ShowcaseScene) -> (SimSession, ShowcasePlayback) {
    let (meta_json, blocks_bytes): (&str, &[u8]) = match scene {
        ShowcaseScene::Transport => (
            include_str!("../../assets/showcases/transport/meta.json"),
            include_bytes!("../../assets/showcases/transport/blocks.bin"),
        ),
        ShowcaseScene::Conversion => (
            include_str!("../../assets/showcases/conversion/meta.json"),
            include_bytes!("../../assets/showcases/conversion/blocks.bin"),
        ),
        ShowcaseScene::Welding => (
            include_str!("../../assets/showcases/welding/meta.json"),
            include_bytes!("../../assets/showcases/welding/blocks.bin"),
        ),
        ShowcaseScene::Pusher => (
            include_str!("../../assets/showcases/pusher/meta.json"),
            include_bytes!("../../assets/showcases/pusher/blocks.bin"),
        ),
        ShowcaseScene::Lifter => (
            include_str!("../../assets/showcases/lifter/meta.json"),
            include_bytes!("../../assets/showcases/lifter/blocks.bin"),
        ),
    };
    let meta = serde_json::from_str::<ShowcaseMetaFile>(meta_json)
        .unwrap_or_else(|error| panic!("invalid showcase {} meta.json: {error}", scene.id()));
    let blocks = decode_blocks(blocks_bytes)
        .unwrap_or_else(|error| panic!("invalid showcase {} blocks.bin: {error}", scene.id()));
    let mut session = SimSession::new();
    session.world = world_from_free_blocks(blocks);
    refresh_static_generated_markers(&mut session.world);
    session.request_continuous_run();
    (session, meta.showcase)
}

/// 进入启动菜单时创建专用渲染资源、等轴相机、真实世界和 UI 背景。
fn setup_start_menu_showcase(
    mut commands: Commands,
    mut assets: ShowcaseRenderAssetDeps,
    config: Res<GameConfig>,
    i18n: Res<I18n>,
    ui_root: Res<UiRootEntity>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let (width, height) = window
        .single()
        .map(showcase_view_size)
        .unwrap_or((1280, 720));
    let view = assets.images.add(new_gameplay_view_image(width, height));
    let mut render_assets = create_world_render_assets(
        &mut assets.meshes,
        &mut assets.materials,
        &mut assets.shadow_proxy_materials,
        &mut assets.ghost_materials,
        &mut assets.portal_materials,
        &mut assets.images,
        &assets.scene_registry,
        &assets.material_registry,
        &assets.stamp_registry,
        &assets.paint_registry,
    );
    render_assets.set_goal_play_visual(true);
    let scene = ShowcaseScene::Transport;
    let (session, playback) = showcase_session(scene);
    let camera_position = Vec3::from_array(playback.camera_position);
    let camera_target = Vec3::from_array(playback.camera_target);
    let (title_key, subtitle_key) = scene.text_keys();

    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            clear_color: ClearColorConfig::Custom(MENU_CLEAR),
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: playback.viewport_height,
            },
            ..OrthographicProjection::default_3d()
        }),
        RenderTarget::Image(view.clone().into()),
        Transform::from_translation(camera_position).looking_at(camera_target, Vec3::Y),
        Hdr,
        Msaa::Off,
        Tonemapping::TonyMcMapface,
        DebandDither::Enabled,
        StartMenuShowcaseCamera,
        StartMenuShowcaseEntity,
        WorldViewCamera,
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 14_000.0,
            color: Color::srgb(1.0, 0.94, 0.84),
            shadow_maps_enabled: config.shadows_enabled,
            ..default()
        },
        Transform::from_xyz(6.0, 10.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        CascadeShadowConfigBuilder {
            num_cascades: 2,
            minimum_distance: 0.15,
            maximum_distance: 18.0,
            first_cascade_far_bound: 7.0,
            overlap_proportion: 0.16,
        }
        .build(),
        StartMenuShowcaseEntity,
    ));
    commands.spawn((
        PointLight {
            intensity: 850.0,
            range: 16.0,
            radius: 3.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(-3.0, 6.5, 4.0),
        StartMenuShowcaseEntity,
    ));

    let mut block_index = BlockEntityIndex::default();
    let mut scene_chunks = SceneChunkMeshes::default();
    rebuild_world_with_runtime_animations(
        &mut commands,
        &mut assets.meshes,
        &session.world,
        &render_assets,
        &HashMap::new(),
        &HashMap::new(),
        AnimationTiming::simulation(SIMULATION_TURN_SECONDS),
        &HashSet::new(),
        &mut block_index,
        &mut scene_chunks,
    );
    commands.insert_resource(render_assets);

    commands.entity(ui_root.0).with_children(|root| {
        root.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(MENU_CLEAR),
            GlobalZIndex(-100),
            Pickable::IGNORE,
            StartMenuShowcaseBackdrop,
            StartMenuShowcaseEntity,
        ))
        .with_children(|backdrop| {
            backdrop.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ImageNode {
                    image: view.clone(),
                    color: Color::WHITE.with_alpha(0.0),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Pickable::IGNORE,
                StartMenuShowcaseImage,
            ));
            backdrop.spawn((
                Node {
                    width: Val::Percent(52.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.075, 0.08, 0.48)),
                Pickable::IGNORE,
            ));
            backdrop
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(28.0),
                        bottom: Val::Px(24.0),
                        padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexEnd,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.08, 0.075, 0.08, 0.68)),
                    Pickable::IGNORE,
                ))
                .with_children(|caption| {
                    caption.spawn((
                        Text::new(i18n.t(title_key)),
                        TextFont {
                            font_size: default_font_size(17.0),
                            weight: FontWeight::SEMIBOLD,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.90, 0.75)),
                        LocalizedText { key: title_key },
                        StartMenuShowcaseCaption::Title,
                        Pickable::IGNORE,
                    ));
                    caption.spawn((
                        Text::new(i18n.t(subtitle_key)),
                        TextFont {
                            font_size: default_font_size(11.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.88, 0.84, 0.78)),
                        LocalizedText { key: subtitle_key },
                        StartMenuShowcaseCaption::Subtitle,
                        Pickable::IGNORE,
                    ));
                });
        });
    });

    commands.insert_resource(StartMenuShowcase {
        session,
        scene,
        playback,
        block_index,
        scene_chunks,
        view,
        previous_powered_wires: HashSet::new(),
        turn_elapsed: 0.0,
        phase_elapsed: 0.0,
        phase: ShowcasePhase::FadeIn,
    });
}

/// 仅在未被其它菜单页或模态面板遮挡时显示橱窗并启用离屏相机。
fn sync_start_menu_showcase_visibility(
    navigation: Res<UiNavigation>,
    busy: Res<SessionBusy>,
    showcase: Option<Res<StartMenuShowcase>>,
    mut cameras: Query<&mut Camera, With<StartMenuShowcaseCamera>>,
    mut backdrops: Query<&mut Visibility, With<StartMenuShowcaseBackdrop>>,
) {
    let Some(_) = showcase else {
        return;
    };
    let visible = navigation.start_menu() == StartMenuPage::Main
        && navigation.active_play()
        && !busy.is_busy();
    for mut backdrop in &mut backdrops {
        *backdrop = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for mut camera in &mut cameras {
        camera.is_active = visible;
    }
}

/// 窗口尺寸变化时同步离屏画面，避免等轴场景被 UI 拉伸。
fn resize_start_menu_showcase_view(
    window: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    showcase: Option<Res<StartMenuShowcase>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(showcase) = showcase else {
        return;
    };
    let Ok(window) = window.single() else {
        return;
    };
    let Some(mut image) = images.get_mut(&showcase.view) else {
        return;
    };
    let (width, height) = showcase_view_size(window);
    if image.width() == width && image.height() == height {
        return;
    }
    image.resize(Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    });
}

/// 按固定节奏推进同一套 ECS 模拟，在材料交付后淡出并恢复初始快照。
fn advance_start_menu_showcase(
    time: Res<Time>,
    navigation: Res<UiNavigation>,
    busy: Res<SessionBusy>,
    showcase: Option<ResMut<StartMenuShowcase>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    render_assets: Option<Res<WorldRenderAssets>>,
    portal_flash_queue: Option<ResMut<PortalFlashQueue>>,
    block_entities: Query<Entity, With<BlockEntity>>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<StartMenuShowcaseCamera>>,
    mut image_nodes: Query<&mut ImageNode, With<StartMenuShowcaseImage>>,
    i18n: Res<I18n>,
    mut captions: Query<
        (&StartMenuShowcaseCaption, &mut LocalizedText, &mut Text),
        Without<StartMenuShowcaseImage>,
    >,
) {
    let (Some(mut showcase), Some(render_assets), Some(mut portal_flash_queue)) =
        (showcase, render_assets, portal_flash_queue)
    else {
        return;
    };
    if navigation.start_menu() != StartMenuPage::Main || !navigation.active_play() || busy.is_busy()
    {
        return;
    }

    let showcase = &mut *showcase;
    let delta = time.delta_secs();
    match showcase.phase {
        ShowcasePhase::FadeIn => {
            showcase.phase_elapsed += delta;
            if showcase.phase_elapsed >= SHOWCASE_FADE_SECONDS {
                showcase.phase = ShowcasePhase::Running;
                showcase.phase_elapsed = 0.0;
            }
        }
        ShowcasePhase::Running => {
            showcase.turn_elapsed += delta;
            if showcase.turn_elapsed >= showcase.playback.turn_seconds {
                showcase.turn_elapsed %= showcase.playback.turn_seconds;
                let before = showcase.session.world.clone();
                let output = showcase.session.simulate_next_turn_with_logging(false);
                let after = showcase.session.world.clone();
                let previous_powered_wires = showcase.previous_powered_wires.clone();
                let mut stats = SimulationStepStats::default();
                let mut scene = SceneRenderMut {
                    commands: &mut commands,
                    meshes: &mut meshes,
                    render_assets: &render_assets,
                    block_index: &mut showcase.block_index,
                    scene_chunks: &mut showcase.scene_chunks,
                };
                apply_turn_output(
                    &before,
                    &after,
                    &output,
                    &previous_powered_wires,
                    SIMULATION_TURN_SECONDS,
                    &mut scene,
                    &mut stats,
                    &mut portal_flash_queue,
                );
                showcase.previous_powered_wires = output.powered_wires.clone();

                if !output.acceptance_sparks.is_empty() {
                    showcase.phase = ShowcasePhase::Completed;
                    showcase.phase_elapsed = 0.0;
                } else if showcase.session.control().turn >= showcase.playback.max_turns {
                    showcase.phase = ShowcasePhase::FadeOut;
                    showcase.phase_elapsed = 0.0;
                }
            }
        }
        ShowcasePhase::Completed => {
            showcase.phase_elapsed += delta;
            if showcase.phase_elapsed >= showcase.playback.completion_hold_seconds {
                showcase.phase = ShowcasePhase::FadeOut;
                showcase.phase_elapsed = 0.0;
            }
        }
        ShowcasePhase::FadeOut => {
            showcase.phase_elapsed += delta;
            if showcase.phase_elapsed >= SHOWCASE_FADE_SECONDS {
                despawn_world(
                    &mut commands,
                    &mut meshes,
                    &block_entities,
                    &mut showcase.block_index,
                    &mut showcase.scene_chunks,
                );
                showcase.scene = match showcase.scene {
                    ShowcaseScene::Transport => ShowcaseScene::Conversion,
                    ShowcaseScene::Conversion => ShowcaseScene::Welding,
                    ShowcaseScene::Welding => ShowcaseScene::Pusher,
                    ShowcaseScene::Pusher => ShowcaseScene::Lifter,
                    ShowcaseScene::Lifter => ShowcaseScene::Transport,
                };
                let (session, playback) = showcase_session(showcase.scene);
                showcase.session = session;
                showcase.playback = playback;
                showcase.previous_powered_wires.clear();
                showcase.turn_elapsed = 0.0;
                showcase.phase_elapsed = 0.0;
                showcase.phase = ShowcasePhase::FadeIn;
                let world = showcase.session.world.clone();
                rebuild_world_with_runtime_animations(
                    &mut commands,
                    &mut meshes,
                    &world,
                    &render_assets,
                    &HashMap::new(),
                    &HashMap::new(),
                    AnimationTiming::simulation(SIMULATION_TURN_SECONDS),
                    &HashSet::new(),
                    &mut showcase.block_index,
                    &mut showcase.scene_chunks,
                );
                let camera_position = Vec3::from_array(showcase.playback.camera_position);
                let camera_target = Vec3::from_array(showcase.playback.camera_target);
                for (mut transform, mut projection) in &mut cameras {
                    *transform = Transform::from_translation(camera_position)
                        .looking_at(camera_target, Vec3::Y);
                    if let Projection::Orthographic(projection) = &mut *projection {
                        projection.scaling_mode = ScalingMode::FixedVertical {
                            viewport_height: showcase.playback.viewport_height,
                        };
                    }
                }
                let (title_key, subtitle_key) = showcase.scene.text_keys();
                for (caption, mut localized, mut text) in &mut captions {
                    localized.key = match caption {
                        StartMenuShowcaseCaption::Title => title_key,
                        StartMenuShowcaseCaption::Subtitle => subtitle_key,
                    };
                    text.0 = i18n.t(localized.key).to_owned();
                }
            }
        }
    }

    let opacity = match showcase.phase {
        ShowcasePhase::FadeIn => (showcase.phase_elapsed / SHOWCASE_FADE_SECONDS).clamp(0.0, 1.0),
        ShowcasePhase::Running => 1.0,
        ShowcasePhase::Completed => 1.0,
        ShowcasePhase::FadeOut => {
            1.0 - (showcase.phase_elapsed / SHOWCASE_FADE_SECONDS).clamp(0.0, 1.0)
        }
    };
    for mut image in &mut image_nodes {
        image.color = Color::WHITE.with_alpha(opacity);
    }
}

/// 给本帧新增的方块和模拟特效补上橱窗归属，便于状态退出时完整清理。
fn mark_start_menu_showcase_entities(
    mut commands: Commands,
    block_entities: Query<Entity, Added<BlockEntity>>,
    effects: Query<
        Entity,
        Or<(
            Added<WeldSpark>,
            Added<WeldBurstParticle>,
            Added<BreakDebrisParticle>,
            Added<LaserBeamBurst>,
        )>,
    >,
) {
    for entity in &block_entities {
        commands.entity(entity).insert(StartMenuShowcaseEntity);
    }
    for entity in &effects {
        commands.entity(entity).insert(StartMenuShowcaseEntity);
    }
}

/// 离开启动菜单时销毁橱窗实体、场景网格、离屏图片和专用渲染资源。
fn teardown_start_menu_showcase(
    mut commands: Commands,
    showcase: Option<ResMut<StartMenuShowcase>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    block_entities: Query<Entity, With<BlockEntity>>,
    showcase_entities: Query<Entity, (With<StartMenuShowcaseEntity>, Without<BlockEntity>)>,
) {
    let Some(mut showcase) = showcase else {
        commands.remove_resource::<WorldRenderAssets>();
        return;
    };
    let showcase = &mut *showcase;
    despawn_world(
        &mut commands,
        &mut meshes,
        &block_entities,
        &mut showcase.block_index,
        &mut showcase.scene_chunks,
    );
    for entity in &showcase_entities {
        commands.entity(entity).despawn();
    }
    images.remove(showcase.view.id());
    commands.remove_resource::<StartMenuShowcase>();
    commands.remove_resource::<WorldRenderAssets>();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 所有内置橱窗都应在各自最大播放回合内完成至少一次材料验收。
    #[test]
    fn bundled_showcases_complete_before_timeout() {
        crate::game::blocks::ensure_fallback_scene_catalog();
        crate::game::blocks::ensure_fallback_material_catalog();

        for scene in [
            ShowcaseScene::Transport,
            ShowcaseScene::Conversion,
            ShowcaseScene::Welding,
            ShowcaseScene::Pusher,
            ShowcaseScene::Lifter,
        ] {
            let (mut session, playback) = showcase_session(scene);
            let mut trace = Vec::new();
            let completed = (0..playback.max_turns).any(|_| {
                let output = session.simulate_next_turn_with_logging(false);
                trace.push(format!(
                    "turn={} materials={:?} welders={:?} weld_points={:?} welds={}",
                    session.control().turn,
                    session
                        .world
                        .blocks()
                        .iter()
                        .filter_map(|(pos, block)| block.kind.is_material().then_some(*pos))
                        .collect::<Vec<_>>(),
                    session
                        .world
                        .blocks()
                        .iter()
                        .filter_map(|(pos, block)| matches!(
                            block.kind,
                            crate::game::blocks::BlockKind::Welder
                                | crate::game::blocks::BlockKind::DownWelder
                        )
                        .then_some((*pos, block.kind, block.facing)))
                        .collect::<Vec<_>>(),
                    session
                        .world
                        .system_blocks()
                        .iter()
                        .filter_map(|(pos, block)| (block.kind
                            == crate::game::blocks::BlockKind::WeldPoint)
                            .then_some(*pos))
                        .collect::<Vec<_>>(),
                    session.world.material_welds().len(),
                ));
                !output.acceptance_sparks.is_empty()
            });
            assert!(
                completed,
                "showcase {} did not complete: {trace:#?}",
                scene.id()
            );
        }
    }
}
