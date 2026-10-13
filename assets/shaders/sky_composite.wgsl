// 全分辨率天空背景与线性云重建；深度为零的全屏三角形只填充远平面。
#import bevy_render::view::View
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

// 与云通道共用相机和线性太阳参数。
struct SkyUniform {
    sun_dir: vec3<f32>, exposure: f32,
    sun_color: vec3<f32>, time: f32,
};
@group(0) @binding(0) var cloud_image: texture_2d<f32>;
@group(0) @binding(1) var cloud_sampler: sampler;
@group(0) @binding(2) var<uniform> view: View;
@group(0) @binding(3) var<uniform> sky: SkyUniform;

// 保持太阳圆盘锐利，在线性 HDR 中合成，交由主相机统一色调映射。
@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let near = view.view_from_clip * vec4<f32>(in.uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 1.0, 1.0);
    let dir = normalize((view.world_from_view * vec4<f32>(near.xyz / near.w, 0.0)).xyz);
    let mu = clamp(dot(normalize(sky.sun_dir), dir), -1.0, 1.0);
    var background = 6.0 * mix(vec3<f32>(0.2, 0.52, 1.0), vec3<f32>(0.8, 0.95, 1.0),
        pow(0.5 + 0.5 * mu, 15.0));
    background += mix(vec3<f32>(3.5), vec3<f32>(0.0), clamp(2.3 * dir.y, 0.0, 1.0));
    background += sky.sun_color * 10000.0 * smoothstep(0.9998, 1.0, mu);
    let cloud = textureSampleLevel(cloud_image, cloud_sampler, in.uv, 0.0);
    return vec4<f32>(cloud.rgb + background * sky.exposure * cloud.a, 1.0);
}
