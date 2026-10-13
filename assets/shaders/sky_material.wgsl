// 体积云：基于 Enscape Cube 的云层与散射思路重写。
// https://www.shadertoy.com/view/4dSBDt — ThomasSchander, CC BY-NC-SA 3.0
// 天气图和体积噪声预生成；观察光线与太阳光线共用密度函数。
#import bevy_render::view::View
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

// 与相机提取的 Rust 参数保持相同布局。
struct SkyUniform {
    sun_dir: vec3<f32>,
    exposure: f32,
    sun_color: vec3<f32>,
    time: f32,
};
@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var<uniform> sky: SkyUniform;
@group(0) @binding(2) var volume_noise: texture_3d<f32>;
@group(0) @binding(3) var weather_noise: texture_2d<f32>;
@group(0) @binding(4) var noise_sampler: sampler;

// 球形云层的物理尺度，单位为米。
const EARTH_RADIUS: f32 = 6300000.0;
const CLOUD_BASE: f32 = 800.0;
const CLOUD_DEPTH: f32 = 600.0;
const VIEW_STEPS: u32 = 64u;
const LIGHT_STEPS: u32 = 6u;

// 平滑体积纹理插值；显式 LOD 使动态步进分支无需隐式导数。
fn noise(p: vec3<f32>) -> f32 {
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return textureSampleLevel(volume_noise, noise_sampler, (floor(p) + u + 0.5) / 32.0, 0.0).r;
}

// 三层旋转噪声供主体轮廓和细节侵蚀共用。
fn fbm(p0: vec3<f32>) -> f32 {
    let m = mat3x3<f32>(0.0, 0.8, 0.6, -0.8, 0.36, -0.48, -0.6, -0.48, 0.64);
    let p1 = m * p0 * 2.02;
    let p2 = m * p1 * 2.03;
    return 0.5 * noise(p0) + 0.25 * noise(p1) + 0.125 * noise(p2);
}

// 两种尺度共用周期天气纹理，保留云团与云间空隙。
fn weather(p: vec2<f32>) -> f32 {
    return textureSampleLevel(weather_noise, noise_sampler, p, 0.0).r;
}

// 避免从两个约 630 万的浮点数相减来计算云层高度。
fn altitude(p: vec3<f32>) -> f32 {
    let offset = dot(p, p) + 2.0 * EARTH_RADIUS * p.y;
    return offset / (sqrt(EARTH_RADIUS * EARTH_RADIUS + offset) + EARTH_RADIUS);
}

// 主视线和阴影采样使用完全相同的天气、轮廓和侵蚀。
fn density(p0: vec3<f32>) -> f32 {
    let h = (altitude(p0) - CLOUD_BASE) / CLOUD_DEPTH;
    if h <= 0.0 || h >= 1.0 { return 0.0; }
    var p = p0 + vec3<f32>(0.0, 0.0, sky.time * 10.3);
    let large_weather = clamp((weather(-0.00005 * p.zx) - 0.18) * 5.0, 0.0, 2.0);
    if large_weather < 0.001 { return 0.0; }
    p.x += sky.time * 8.3;
    var coverage = large_weather * max(0.0, weather(0.0002 * p.zx) - 0.36) / 0.64;
    // 显式反向坡度，避免 reversed-edge smoothstep 的未定义行为。
    coverage *= smoothstep(0.0, 0.5, h) * (1.0 - smoothstep(0.5, 1.0, h));
    let shape = pow(max(coverage, 0.0), 0.3 + 1.5 * smoothstep(0.2, 0.5, h));
    p.x += sky.time * 12.3;
    var d = max(0.0, shape - 0.7 * fbm(p * 0.01));
    if d <= 0.0 { return 0.0; }
    p.y += sky.time * 15.2;
    d = max(0.0, d - 0.2 * fbm(p * 0.05));
    return large_weather * 0.12 * min(1.0, 5.0 * d);
}

// 从固定观测高度求云壳出口，使用稳定形式避免近地浮点消减。
fn shell_distance(dir: vec3<f32>, height: f32) -> f32 {
    let b = (EARTH_RADIUS + 3.0) * dir.y;
    let c = (3.0 - height) * (2.0 * EARTH_RADIUS + 3.0 + height);
    return -c / (b + sqrt(b * b - c));
}

// 半分辨率输出预乘云辐射与透射率；太阳和天空背景在合成时全分辨率计算。
@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let near = view.view_from_clip * vec4<f32>(in.uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 1.0, 1.0);
    let dir = normalize((view.world_from_view * vec4<f32>(near.xyz / near.w, 0.0)).xyz);
    if dir.y <= 0.015 { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
    let sun = normalize(sky.sun_dir);
    let mu = dot(sun, dir);
    // Enscape Cube 的数值 Mie 相函数拟合。
    let phase_offset = mu + 0.8194068;
    let phase = dot(exp(vec4<f32>(-65.0 * mu - 55.0, -83.70334 * phase_offset * phase_offset,
        7.810083 * mu, -4.552125e-12 * mu)),
        vec4<f32>(9.805233e-6, 0.1388198, 0.002054747, 0.02600563));
    let start = shell_distance(dir, CLOUD_BASE);
    let end = shell_distance(dir, CLOUD_BASE + CLOUD_DEPTH);
    let step = (end - start) / f32(VIEW_STEPS);
    // 足量中点采样避免低分辨率重建放大规则抖动纹理。
    var transmittance = 1.0;
    var color = vec3<f32>(0.0);
    for (var i = 0u; i < VIEW_STEPS; i += 1u) {
        let p = vec3<f32>(0.0, 3.0, 0.0) + dir * (start + (f32(i) + 0.5) * step);
        let d = density(p);
        if d > 0.001 {
            let h = clamp((altitude(p) - CLOUD_BASE) / CLOUD_DEPTH, 0.0, 1.0);
            let ambient = (0.5 + 0.6 * h) * vec3<f32>(0.2, 0.5, 1.0) * 6.5
                + vec3<f32>(0.8) * max(0.0, 1.0 - 2.0 * h);
            // 与观察光线共用密度，计算太阳光学厚度和多重散射。
            let light_step = CLOUD_DEPTH / f32(LIGHT_STEPS);
            var optical_depth = 0.0;
            for (var j = 0u; j < LIGHT_STEPS; j += 1u) {
                optical_depth += density(p + sun * (f32(j) + 0.5) * light_step) * light_step;
            }
            let scatter = mix(0.008, 1.0, 1.0 - smoothstep(0.0, 0.96, mu));
            let beer = exp(-optical_depth) + 0.5 * scatter * exp(-0.1 * optical_depth)
                + 0.4 * scatter * exp(-0.02 * optical_depth);
            let powder = mix(0.05 + 1.5 * pow(min(1.0, d * 8.5), 0.3 + 5.5 * h),
                1.0, clamp(optical_depth / light_step * 0.4, 0.0, 1.0));
            let direct = vec3<f32>(1.0, 0.9, 0.6) * sky.sun_color * 750.0
                * phase * beer * powder;
            let transmission = exp(-d * step);
            color += transmittance * (ambient + direct) * (1.0 - transmission);
            transmittance *= transmission;
            if transmittance < 0.01 { break; }
        }
    }
    let horizon = smoothstep(0.015, 0.06, dir.y);
    return vec4<f32>(color * sky.exposure * horizon, mix(1.0, transmittance, horizon));
}
