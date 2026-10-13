//! 半分辨率体积云与背景合成；仅在程序化天空启用的相机上执行。

use bevy::core_pipeline::{
    Core3dSystems, FullscreenShader,
    core_3d::{CORE_3D_DEPTH_FORMAT, main_opaque_pass_3d, main_transparent_pass_3d},
    schedule::Core3d,
};
use bevy::prelude::*;
use bevy::render::{
    RenderApp, RenderStartup,
    extract_component::{
        ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
        UniformComponentPlugin,
    },
    render_resource::{
        binding_types::{sampler, texture_2d, texture_3d, uniform_buffer},
        *,
    },
    renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery},
    view::{ViewDepthTexture, ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms},
};

use super::skybox::resolved_sun_direction;
use crate::shared::save::PuzzleLighting;

/// 随相机提取到渲染世界的太阳、曝光和连续动画时间。
#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
pub(super) struct ProceduralSky {
    pub sun_dir: Vec3,
    exposure: f32,
    sun_color: Vec3,
    pub time: f32,
}

impl ProceduralSky {
    /// 初建天空和关卡光照变化共用同一组线性光照参数。
    pub fn from_lighting(lighting: &PuzzleLighting) -> Self {
        let color = lighting.color.to_linear();
        Self {
            sun_dir: resolved_sun_direction(lighting.direction),
            exposure: 0.055,
            sun_color: Vec3::new(color.red, color.green, color.blue)
                * (lighting.illuminance / 9500.0).max(0.0),
            time: 0.0,
        }
    }
}

/// 注册独立天空渲染通道，使用 Bevy 的相机调度和 GPU 抽象。
pub(super) struct ProceduralSkyPlugin;

impl Plugin for ProceduralSkyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<ProceduralSky>::default(),
            UniformComponentPlugin::<ProceduralSky>::default(),
        ))
        .add_systems(PostUpdate, advance_sky_time);
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .add_systems(RenderStartup, prepare_sky_pipeline)
            .add_systems(
                Core3d,
                render_procedural_sky
                    .in_set(Core3dSystems::MainPass)
                    .after(main_opaque_pass_3d)
                    .before(main_transparent_pass_3d),
            );
    }
}

/// 连续时间不依赖绘制次数；相机限帧时天空通道跟随相机一起跳帧。
fn advance_sky_time(time: Res<Time>, mut skies: Query<&mut ProceduralSky>) {
    for mut sky in &mut skies {
        sky.time = time.elapsed_secs_wrapped();
    }
}

/// 两个固定 HDR 管线、可重复采样的噪声纹理和线性重建采样器。
#[derive(Resource)]
struct SkyPipeline {
    cloud_layout: BindGroupLayoutDescriptor,
    composite_layout: BindGroupLayoutDescriptor,
    cloud_pipeline: CachedRenderPipelineId,
    composite_pipeline: CachedRenderPipelineId,
    volume: TextureView,
    weather: TextureView,
    noise_sampler: Sampler,
    reconstruction_sampler: Sampler,
}

/// 一次性生成周期噪声并创建天空管线，避免把噪声生成放进光线步进。
fn prepare_sky_pipeline(
    mut commands: Commands,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    asset_server: Res<AssetServer>,
    fullscreen: Res<FullscreenShader>,
    pipelines: Res<PipelineCache>,
) {
    let hash = |x: u32, y: u32, z: u32| -> f32 {
        let mut n = x.wrapping_mul(1597334677)
            ^ y.wrapping_mul(3812015801)
            ^ z.wrapping_mul(2798796415)
            ^ 0x9e3779b9;
        n = (n ^ (n >> 16)).wrapping_mul(2246822519);
        n = (n ^ (n >> 13)).wrapping_mul(3266489917);
        (n ^ (n >> 16)) as f32 / u32::MAX as f32
    };
    let mut volume = Vec::with_capacity(32 * 32 * 32);
    for z in 0..32 {
        for y in 0..32 {
            for x in 0..32 {
                volume.push((hash(x, y, z) * 255.0) as u8);
            }
        }
    }
    // 天气图使用周期多尺度噪声，边缘重复不会出现纹理接缝。
    let mut weather = Vec::with_capacity(256 * 256);
    for y in 0..256 {
        for x in 0..256 {
            let mut value = 0.0;
            let mut amplitude = 0.5;
            for cells in [4, 8, 16, 32] {
                let p = Vec2::new(x as f32, y as f32) * (cells as f32 / 256.0);
                let i = p.floor().as_uvec2();
                let f = p.fract();
                let u = f * f * (Vec2::splat(3.0) - 2.0 * f);
                let a = hash(i.x % cells, i.y % cells, cells);
                let b = hash((i.x + 1) % cells, i.y % cells, cells);
                let c = hash(i.x % cells, (i.y + 1) % cells, cells);
                let d = hash((i.x + 1) % cells, (i.y + 1) % cells, cells);
                value +=
                    amplitude * ((a + (b - a) * u.x) * (1.0 - u.y) + (c + (d - c) * u.x) * u.y);
                amplitude *= 0.5;
            }
            weather.push((value / 0.9375 * 255.0) as u8);
        }
    }
    let mut views = Vec::new();
    for (label, dimension, size, data) in [
        (
            "sky_volume_noise",
            TextureDimension::D3,
            Extent3d {
                width: 32,
                height: 32,
                depth_or_array_layers: 32,
            },
            volume,
        ),
        (
            "sky_weather",
            TextureDimension::D2,
            Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            },
            weather,
        ),
    ] {
        views.push(
            device
                .create_texture_with_data(
                    &queue,
                    &TextureDescriptor {
                        label: Some(label),
                        size,
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension,
                        format: TextureFormat::R8Unorm,
                        usage: TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    },
                    TextureDataOrder::LayerMajor,
                    &data,
                )
                .create_view(&default()),
        );
    }
    let cloud_layout = BindGroupLayoutDescriptor::new(
        "sky_cloud_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                uniform_buffer::<ViewUniform>(true),
                uniform_buffer::<ProceduralSky>(true),
                texture_3d(TextureSampleType::Float { filterable: true }),
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
            ),
        ),
    );
    let composite_layout = BindGroupLayoutDescriptor::new(
        "sky_composite_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<ViewUniform>(true),
                uniform_buffer::<ProceduralSky>(true),
            ),
        ),
    );
    let mut ids = Vec::new();
    for (label, layout, shader, depth) in [
        (
            "sky_clouds",
            &cloud_layout,
            "shaders/sky_material.wgsl",
            None,
        ),
        (
            "sky_composite",
            &composite_layout,
            "shaders/sky_composite.wgsl",
            Some(DepthStencilState {
                format: CORE_3D_DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(CompareFunction::GreaterEqual),
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
        ),
    ] {
        ids.push(pipelines.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some(label.into()),
            layout: vec![layout.clone()],
            vertex: fullscreen.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: asset_server.load(shader),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::Rgba16Float,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            depth_stencil: depth,
            ..default()
        }));
    }
    commands.insert_resource(SkyPipeline {
        cloud_layout,
        composite_layout,
        cloud_pipeline: ids[0],
        composite_pipeline: ids[1],
        volume: views.remove(0),
        weather: views.remove(0),
        noise_sampler: device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..default()
        }),
        reconstruction_sampler: device.create_sampler(&SamplerDescriptor {
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..default()
        }),
    });
}

/// 缓存单个游玩相机的半分辨率纹理，窗口尺寸变化时才重新分配。
#[derive(Default)]
struct SkyTargetCache {
    target: Option<(UVec2, TextureView)>,
}

/// 在不透明物体之后、透明物体之前写入远平面背景，保留 HDR 与场景深度。
fn render_procedural_sky(
    view: ViewQuery<(
        &ViewTarget,
        &ViewDepthTexture,
        &ViewUniformOffset,
        &DynamicUniformIndex<ProceduralSky>,
    )>,
    pipeline: Option<Res<SkyPipeline>>,
    pipelines: Res<PipelineCache>,
    views: Res<ViewUniforms>,
    skies: Res<ComponentUniforms<ProceduralSky>>,
    mut cache: Local<SkyTargetCache>,
    mut ctx: RenderContext,
) {
    let Some(pipeline) = pipeline else {
        return;
    };
    let (Some(cloud_pipeline), Some(composite_pipeline), Some(view_binding), Some(sky_binding)) = (
        pipelines.get_render_pipeline(pipeline.cloud_pipeline),
        pipelines.get_render_pipeline(pipeline.composite_pipeline),
        views.uniforms.binding(),
        skies.uniforms().binding(),
    ) else {
        return;
    };
    let (target, depth, view_offset, sky_index) = view.into_inner();
    let size = target.main_texture().size();
    let half = UVec2::new(
        size.width.div_ceil(2).max(1),
        size.height.div_ceil(2).max(1),
    );
    if cache.target.as_ref().is_none_or(|(old, _)| *old != half) {
        let texture = ctx
            .render_device()
            .create_texture(&TextureDescriptor {
                label: Some("sky_half_resolution"),
                size: Extent3d {
                    width: half.x,
                    height: half.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&default());
        cache.target = Some((half, texture));
    }
    let (_, texture) = cache.target.as_ref().unwrap();
    let composite_group = ctx.render_device().create_bind_group(
        "sky_composite_bind_group",
        &pipelines.get_bind_group_layout(&pipeline.composite_layout),
        &BindGroupEntries::sequential((
            texture,
            &pipeline.reconstruction_sampler,
            view_binding.clone(),
            sky_binding.clone(),
        )),
    );
    let cloud_group = ctx.render_device().create_bind_group(
        "sky_cloud_bind_group",
        &pipelines.get_bind_group_layout(&pipeline.cloud_layout),
        &BindGroupEntries::sequential((
            view_binding,
            sky_binding,
            &pipeline.volume,
            &pipeline.weather,
            &pipeline.noise_sampler,
        )),
    );
    {
        let mut pass = ctx
            .command_encoder()
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("sky_clouds_half_resolution"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: texture,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(default()),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        pass.set_pipeline(cloud_pipeline);
        pass.set_bind_group(0, &cloud_group, &[view_offset.offset, sky_index.index()]);
        pass.draw(0..3, 0..1);
    }
    let mut pass = ctx
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label: Some("sky_background_composite"),
            color_attachments: &[Some(target.get_color_attachment())],
            depth_stencil_attachment: Some(depth.get_attachment(StoreOp::Store)),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    pass.set_pipeline(composite_pipeline);
    pass.set_bind_group(
        0,
        &composite_group,
        &[view_offset.offset, sky_index.index()],
    );
    pass.draw(0..3, 0..1);
}

/// 原生 GPU 验证实际管线、远平面遮挡与窗口尺寸变化，CI 可按需执行。
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use bevy::audio::AudioPlugin;
    use bevy::camera::{Hdr, RenderTarget};
    use bevy::core_pipeline::tonemapping::Tonemapping;
    use bevy::render::{
        RenderPlugin,
        pipelined_rendering::PipelinedRenderingPlugin,
        view::screenshot::{Screenshot, ScreenshotCaptured},
    };
    use bevy::window::ExitCondition;
    use bevy::winit::WinitPlugin;

    /// 将 GPU 回读图像保存在测试世界，验证实际像素而非只有编译结果。
    #[derive(Resource, Default)]
    struct Captures(Vec<Image>);

    /// 无窗口绘制云与前景，确认合成不覆盖物体且 resize 后仍有画面。
    #[test]
    #[ignore = "需要原生 GPU；cargo test --lib sky_gpu_smoke -- --ignored --nocapture"]
    fn sky_gpu_smoke() {
        let mut app = App::new();
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .set(RenderPlugin {
                    synchronous_pipeline_compilation: true,
                    ..default()
                })
                .disable::<WinitPlugin>()
                .disable::<AudioPlugin>()
                .disable::<PipelinedRenderingPlugin>(),
        )
        .add_plugins(ProceduralSkyPlugin)
        .init_resource::<Captures>();
        app.finish();
        app.cleanup();
        let image = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(Image::new_target_texture(
                640,
                360,
                TextureFormat::Rgba8UnormSrgb,
                None,
            ));
        let camera_transform = Transform::default().looking_to(Vec3::new(0.0, 0.55, -1.0), Vec3::Y);
        let camera = app
            .world_mut()
            .spawn((
                Camera3d::default(),
                Hdr,
                Msaa::Off,
                Tonemapping::TonyMcMapface,
                RenderTarget::Image(image.clone().into()),
                camera_transform,
                ProceduralSky::from_lighting(&PuzzleLighting::default()),
            ))
            .id();
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(Cuboid::from_length(0.7));
        let material = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.0, 0.5),
                unlit: true,
                ..default()
            });
        app.world_mut().spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(camera_transform.forward() * 3.0),
        ));
        for (width, height) in [(640, 360), (801, 451)] {
            app.world_mut()
                .resource_mut::<Assets<Image>>()
                .get_mut(&image)
                .unwrap()
                .resize(Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                });
            for _ in 0..40 {
                app.update();
                app.world()
                    .resource::<RenderDevice>()
                    .poll(PollType::Wait {
                        submission_index: None,
                        timeout: None,
                    })
                    .unwrap();
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            let render = app.sub_app(RenderApp);
            let pipeline = render.world().resource::<SkyPipeline>();
            let pipelines = render.world().resource::<PipelineCache>();
            for id in [pipeline.cloud_pipeline, pipeline.composite_pipeline] {
                assert!(
                    pipelines.get_render_pipeline(id).is_some(),
                    "{:?}",
                    pipelines.get_render_pipeline_state(id)
                );
            }
            app.world_mut()
                .spawn(Screenshot::image(image.clone()))
                .observe(
                    |event: On<ScreenshotCaptured>, mut captures: ResMut<Captures>| {
                        captures.0.push(event.image.clone());
                    },
                );
            let count = app.world().resource::<Captures>().0.len();
            for _ in 0..20 {
                app.update();
                app.world()
                    .resource::<RenderDevice>()
                    .poll(PollType::Wait {
                        submission_index: None,
                        timeout: None,
                    })
                    .unwrap();
                if app.world().resource::<Captures>().0.len() > count {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            let captures = app.world().resource::<Captures>();
            assert!(captures.0.len() > count, "GPU screenshot timed out");
            let rgba = captures
                .0
                .last()
                .unwrap()
                .clone()
                .try_into_dynamic()
                .unwrap()
                .to_rgba8();
            assert_eq!(rgba.dimensions(), (width, height));
            let center = rgba.get_pixel(width / 2, height / 2);
            assert!(
                u16::from(center[0]) > 2 * u16::from(center[1]) && center[2] > center[1],
                "foreground covered: {center:?}"
            );
            let corner = rgba.get_pixel(width / 4, height / 4);
            assert!(corner[0] > 10 && corner[2] > 30, "sky missing: {corner:?}");
            if let Some(path) = std::env::var_os("OIF_SKY_TEST_IMAGE") {
                rgba.save(path).unwrap();
            }
        }
        // 主世界移除后，提取世界也必须清除天空，避免关闭后继续绘制。
        app.world_mut().entity_mut(camera).remove::<ProceduralSky>();
        app.update();
        let render = app.sub_app_mut(RenderApp);
        let world = render.world_mut();
        let mut skies = world.query::<&ProceduralSky>();
        assert_eq!(skies.iter(world).count(), 0);
    }
}
