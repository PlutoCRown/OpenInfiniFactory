//! 主世界与渲染世界共用的首帧回执，按场景替换独立追踪。

use super::{BlockIconAssets, GoalGhostMaterial, PortalMaterial, ShadowProxyMaterial};
use bevy::asset::{AssetId, UntypedAssetId};
use bevy::camera::RenderTarget;
use bevy::core_pipeline::{
    schedule::{Core2d, Core3d},
    upscaling::{ViewUpscalingPipeline, upscaling},
};
use bevy::light::{EnvironmentMapLight, Skybox};
use bevy::pbr::PreparedMaterial;
use bevy::prelude::*;
use bevy::render::{
    Render, RenderApp, RenderSystems,
    erased_render_asset::ErasedRenderAssets,
    extract_component::{ExtractComponent, ExtractComponentPlugin},
    mesh::RenderMesh,
    render_asset::RenderAssets,
    render_resource::{CachedPipelineState, PipelineCache, PollType},
    renderer::{RenderDevice, RenderQueue, ViewQuery},
    texture::GpuImage,
    view::ViewTarget,
};
use bevy::shader::ShaderCacheError;
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

/// 正在等待本次场景的资源和绘制管线。
const FRAME_WAITING: u8 = 0;
/// 首帧绘制命令已记录，尚未登记 GPU 完成回调。
const FRAME_RECORDED: u8 = 1;
/// 命令已提交，正在等待 GPU 完成。
const FRAME_SUBMITTED: u8 = 2;
/// GPU 已完成首帧。
const FRAME_READY: u8 = 3;
/// GPU 或管线出现不可恢复错误。
const FRAME_FAILED: u8 = 4;

/// 相机本次世界的回执；旧 GPU 回调只持有旧 Arc，不会放行新世界。
#[derive(Component, Clone, Default, ExtractComponent)]
pub struct SceneFrameReady {
    status: Arc<AtomicU8>,
    assets: Arc<SceneFrameAssets>,
    captured: bool,
}

impl SceneFrameReady {
    /// GPU 已完成首帧提交，离屏画面可供 UI 使用。
    pub fn is_ready(&self) -> bool {
        self.status.load(Ordering::Acquire) == FRAME_READY
    }
    /// 管线产生不可恢复错误，停止等待并由会话报告失败。
    pub fn is_failed(&self) -> bool {
        self.status.load(Ordering::Acquire) == FRAME_FAILED
    }
}

/// 首帧所需的 mesh、材质及相机纹理，只在世界替换时收集一次。
#[derive(Default)]
struct SceneFrameAssets {
    meshes: Vec<AssetId<Mesh>>,
    materials: Vec<UntypedAssetId>,
    images: Vec<AssetId<Image>>,
}

/// 标记当前 App 是否有渲染世界，无头模拟不等待 GPU。
#[derive(Resource)]
struct HasSceneRenderer(bool);

/// 管理跨世界提取、资源就绪与 GPU 完成回调的生命周期。
pub(super) struct SceneReadinessPlugin;

impl Plugin for SceneReadinessPlugin {
    fn build(&self, app: &mut App) {
        let has_renderer = app.get_sub_app(RenderApp).is_some();
        app.insert_resource(HasSceneRenderer(has_renderer))
            .add_plugins(ExtractComponentPlugin::<SceneFrameReady>::default())
            .add_systems(PostUpdate, capture_scene_frame_assets);
        if let Some(render) = app.get_sub_app_mut(RenderApp) {
            render
                .add_systems(Core3d, record_ready_scene_frame.after(upscaling))
                .add_systems(Core2d, record_ready_scene_frame.after(upscaling))
                .add_systems(
                    Render,
                    acknowledge_scene_frame.in_set(RenderSystems::Cleanup),
                );
        }
    }
}

/// 主世界命令落地后冻结本次世界的渲染依赖，不复制整份渲染资源。
fn capture_scene_frame_assets(
    backend: Res<HasSceneRenderer>,
    icons: Option<Res<BlockIconAssets>>,
    cover: Option<Res<crate::game::session::SessionBusyCover>>,
    meshes: Query<(
        &Mesh3d,
        Option<&MeshMaterial3d<StandardMaterial>>,
        Option<&MeshMaterial3d<GoalGhostMaterial>>,
        Option<&MeshMaterial3d<PortalMaterial>>,
        Option<&MeshMaterial3d<ShadowProxyMaterial>>,
    )>,
    mut cameras: Query<(
        &mut SceneFrameReady,
        &RenderTarget,
        Option<&Skybox>,
        Option<&EnvironmentMapLight>,
        Option<&Camera3d>,
    )>,
) {
    for (mut frame, target, sky, env, world_camera) in &mut cameras {
        if frame.captured {
            continue;
        }
        if !backend.0 {
            frame.status.store(FRAME_READY, Ordering::Release);
            frame.captured = true;
            continue;
        }
        let mut assets = SceneFrameAssets::default();
        if let Some(image) = cover.as_ref().and_then(|cover| cover.image.as_ref()) {
            assets.images.push(image.id());
        }
        if let Some(icons) = icons.as_ref() {
            assets.images.extend(
                icons
                    .icons
                    .values()
                    .chain(icons.paints.values())
                    .map(|image| image.id()),
            );
            assets
                .images
                .extend(icons.selection.as_ref().map(|image| image.id()));
            assets
                .images
                .extend(icons.light_panel.as_ref().map(|image| image.id()));
        }
        // UI 遮罩只等待自身纹理和绘制通道，菜单橱窗的动态 mesh 不属于它。
        if world_camera.is_some() {
            for (mesh, standard, ghost, portal, shadow) in &meshes {
                assets.meshes.push(mesh.id());
                assets.materials.extend(standard.map(|m| m.id().untyped()));
                assets.materials.extend(ghost.map(|m| m.id().untyped()));
                assets.materials.extend(portal.map(|m| m.id().untyped()));
                assets.materials.extend(shadow.map(|m| m.id().untyped()));
            }
        }
        if let RenderTarget::Image(image) = target {
            assets.images.push(image.handle.id());
        }
        if let Some(image) = sky.and_then(|sky| sky.image.as_ref()) {
            assets.images.push(image.id());
        }
        if let Some(env) = env {
            assets.images.push(env.diffuse_map.id());
            assets.images.push(env.specular_map.id());
        }
        assets.meshes.sort_unstable();
        assets.meshes.dedup();
        assets.materials.sort_unstable();
        assets.materials.dedup();
        assets.images.sort_unstable();
        assets.images.dedup();
        frame.assets = Arc::new(assets);
        frame.captured = true;
    }
}

/// 最终输出通道后检查 GPU 资产与管线，只有真正可绘制的帧进入提交阶段。
fn record_ready_scene_frame(
    view: ViewQuery<(&SceneFrameReady, &ViewTarget, &ViewUpscalingPipeline)>,
    pipelines: Res<PipelineCache>,
    meshes: Res<RenderAssets<RenderMesh>>,
    materials: Res<ErasedRenderAssets<PreparedMaterial>>,
    images: Res<RenderAssets<GpuImage>>,
) {
    let (frame, target, _) = view.into_inner();
    if !frame.captured || frame.status.load(Ordering::Acquire) != FRAME_WAITING {
        return;
    }
    if pipelines.pipelines().any(|pipeline| {
        matches!(
            &pipeline.state,
            CachedPipelineState::Err(
                ShaderCacheError::ProcessShaderError(_) | ShaderCacheError::CreateShaderModule(_)
            )
        )
    }) {
        frame.status.store(FRAME_FAILED, Ordering::Release);
        return;
    }
    if pipelines.waiting_pipelines().next().is_some()
        || target.out_texture_color_attachment(None).is_none()
        || frame
            .assets
            .meshes
            .iter()
            .any(|id| meshes.get(*id).is_none())
        || frame
            .assets
            .materials
            .iter()
            .any(|id| materials.get(*id).is_none())
        || frame
            .assets
            .images
            .iter()
            .any(|id| images.get(*id).is_none())
    {
        return;
    }
    frame.status.store(FRAME_RECORDED, Ordering::Release);
}

/// Render 已提交命令后注册非阻塞回调，避免将 CPU 记录完成误判成 GPU 就绪。
fn acknowledge_scene_frame(
    frames: Query<&SceneFrameReady>,
    queue: Res<RenderQueue>,
    device: Res<RenderDevice>,
) {
    let mut pending = false;
    for frame in &frames {
        if frame
            .status
            .compare_exchange(
                FRAME_RECORDED,
                FRAME_SUBMITTED,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
        {
            let status = frame.status.clone();
            queue.on_submitted_work_done(move || {
                let _ = status.compare_exchange(
                    FRAME_SUBMITTED,
                    FRAME_READY,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                );
            });
        }
        pending |= frame.status.load(Ordering::Acquire) == FRAME_SUBMITTED;
    }
    if pending {
        if let Err(error) = device.poll(PollType::Poll) {
            error!("first scene frame GPU poll failed: {error}");
            for frame in &frames {
                if frame.status.load(Ordering::Acquire) == FRAME_SUBMITTED {
                    frame.status.store(FRAME_FAILED, Ordering::Release);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 被替换场景的异步回调不能解除新场景的等待。
    #[test]
    fn old_gpu_receipt_cannot_release_replaced_scene() {
        let mut frame = SceneFrameReady::default();
        let old = frame.clone();
        frame = SceneFrameReady::default();
        old.status.store(FRAME_READY, Ordering::Release);
        assert!(old.is_ready());
        assert!(!frame.is_ready());
    }

    /// 无头 App 无渲染后端时不等待 GPU，也不需要伪造帧数。
    #[test]
    fn headless_scene_does_not_wait_for_a_gpu_receipt() {
        let mut app = App::new();
        app.add_plugins(SceneReadinessPlugin);
        let camera = app
            .world_mut()
            .spawn((SceneFrameReady::default(), RenderTarget::default()))
            .id();
        app.update();
        assert!(
            app.world()
                .get::<SceneFrameReady>(camera)
                .unwrap()
                .is_ready()
        );
    }

    /// 窗口调整后必须撤销旧回执，并越过限帧立即绘制新纹理。
    #[test]
    fn resized_view_cannot_reuse_old_ready_frame_or_skip_rendering() {
        use crate::game::cameras::{
            GameplayCamera, GameplayRenderThrottle, GameplayViewImage, new_gameplay_view_image,
            sync_gameplay_render_rate, sync_gameplay_view_image_size,
        };
        use crate::shared::config::GameConfig;
        use bevy::window::PrimaryWindow;
        let mut app = App::new();
        app.init_resource::<Assets<Image>>()
            .init_resource::<GameConfig>()
            .init_resource::<Time<Real>>()
            .init_resource::<GameplayRenderThrottle>()
            .add_systems(
                Update,
                (sync_gameplay_view_image_size, sync_gameplay_render_rate).chain(),
            );
        let image = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(new_gameplay_view_image(640, 360));
        app.insert_resource(GameplayViewImage(image.clone()));
        let window = app
            .world_mut()
            .spawn((
                Window {
                    resolution: (640, 360).into(),
                    ..default()
                },
                PrimaryWindow,
            ))
            .id();
        let frame = SceneFrameReady::default();
        frame.status.store(FRAME_READY, Ordering::Release);
        let old_frame = frame.clone();
        let camera = app
            .world_mut()
            .spawn((Camera::default(), GameplayCamera, frame))
            .id();
        app.update();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set(800.0, 450.0);
        app.world_mut().get_mut::<Camera>(camera).unwrap().is_active = false;
        app.update();
        assert!(old_frame.is_ready());
        assert!(
            !app.world()
                .get::<SceneFrameReady>(camera)
                .unwrap()
                .is_ready()
        );
        assert!(app.world().get::<Camera>(camera).unwrap().is_active);
        let images = app.world().resource::<Assets<Image>>();
        assert_eq!(
            (
                images.get(&image).unwrap().width(),
                images.get(&image).unwrap().height()
            ),
            (800, 450)
        );
    }
}
