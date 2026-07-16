// 天空 + 体积云：改编自 Shadertoy「Enscape Cube」by ThomasSchander
// https://www.shadertoy.com/view/4dSBDt  （仅保留 skyRay / clouds，去掉海面与黄方块）
// License: CC BY-NC-SA 3.0
// 游戏向：步进与噪声层数已压过，避免全屏体积云拖垮帧率
#import bevy_pbr::{
    mesh_view_bindings::{globals, view},
    forward_io::VertexOutput,
}

struct SkyUniform {
    sun_dir: vec3<f32>,
    exposure: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> sky: SkyUniform;

const EARTH_RADIUS: f32 = 6300e3;
const CLOUD_START: f32 = 800.0;
const CLOUD_HEIGHT: f32 = 600.0;
const SUN_POWER: vec3<f32> = vec3<f32>(1.0, 0.9, 0.6) * 750.0;
const NB_SAMPLE: i32 = 10;
const NB_SAMPLE_LIGHT: i32 = 3;

fn hash11(n: f32) -> f32 {
    return fract(sin(n) * 43758.5453);
}

fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn noise2(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash21(i), hash21(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash21(i + vec2<f32>(0.0, 1.0)), hash21(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

fn noise3(x: vec3<f32>) -> f32 {
    let p = floor(x);
    var f = fract(x);
    f = f * f * (3.0 - 2.0 * f);
    let n = p.x + p.y * 157.0 + 113.0 * p.z;
    return mix(
        mix(
            mix(hash11(n + 0.0), hash11(n + 1.0), f.x),
            mix(hash11(n + 157.0), hash11(n + 158.0), f.x),
            f.y
        ),
        mix(
            mix(hash11(n + 113.0), hash11(n + 114.0), f.x),
            mix(hash11(n + 270.0), hash11(n + 271.0), f.x),
            f.y
        ),
        f.z
    );
}

fn fbm2(p0: vec2<f32>) -> f32 {
    let m = mat2x2<f32>(1.6, 1.2, -1.2, 1.6);
    var p = p0;
    var f = 0.0;
    var a = 0.5;
    for (var i = 0; i < 3; i = i + 1) {
        f = f + a * noise2(p);
        p = m * p;
        a = a * 0.5;
    }
    return f;
}

fn fbm3(p0: vec3<f32>) -> f32 {
    let m = mat3x3<f32>(
        0.00, 0.80, 0.60,
        -0.80, 0.36, -0.48,
        -0.60, -0.48, 0.64
    );
    var p = p0;
    var f = 0.5 * noise3(p);
    p = m * p * 2.02;
    f = f + 0.25 * noise3(p);
    p = m * p * 2.03;
    f = f + 0.125 * noise3(p);
    return f;
}

/// 轻度域扭曲：比双层扭曲便宜，形状仍不规则
fn weather_map(p0: vec2<f32>) -> f32 {
    let warp = vec2<f32>(fbm2(p0), fbm2(p0 + vec2<f32>(5.2, 1.3)));
    return fbm2(p0 + 1.8 * warp);
}

fn intersect_sphere(origin: vec3<f32>, dir: vec3<f32>, sphere_pos: vec3<f32>, sphere_rad: f32) -> f32 {
    let oc = origin - sphere_pos;
    let b = 2.0 * dot(dir, oc);
    let c = dot(oc, oc) - sphere_rad * sphere_rad;
    let disc = b * b - 4.0 * c;
    if disc < 0.0 {
        return -1.0;
    }
    var q = (-b + select(sqrt(disc), -sqrt(disc), b < 0.0)) / 2.0;
    var t0 = q;
    var t1 = c / q;
    if t0 > t1 {
        let temp = t0;
        t0 = t1;
        t1 = temp;
    }
    if t1 < 0.0 {
        return -1.0;
    }
    return select(t0, t1, t0 < 0.0);
}

fn clouds(p_in: vec3<f32>, cloud_height_out: ptr<function, f32>, detailed: bool) -> f32 {
    var p = p_in;
    let atmo_height = length(p - vec3<f32>(0.0, -EARTH_RADIUS, 0.0)) - EARTH_RADIUS;
    let cloud_height = clamp((atmo_height - CLOUD_START) / CLOUD_HEIGHT, 0.0, 1.0);
    *cloud_height_out = cloud_height;

    p.z = p.z + globals.time * 10.3;
    let large_n = weather_map(-0.00032 * p.zx);
    // 覆盖率保持偏稀；阈值略放宽，减少“碎渣”
    let large_weather = smoothstep(0.48, 0.74, large_n) * 1.55;
    if large_weather <= 0.001 {
        return 0.0;
    }

    p.x = p.x + globals.time * 8.3;
    var weather = large_weather;
    if detailed {
        let detail_n = weather_map(0.0014 * p.zx + vec2<f32>(17.0, 3.0));
        weather = weather * mix(0.55, 1.0, smoothstep(0.42, 0.68, detail_n));
    }
    weather = weather * smoothstep(0.05, 0.40, cloud_height) * smoothstep(1.0, 0.55, cloud_height);
    let cloud_shape = pow(max(weather, 0.0), 0.7 + 0.85 * smoothstep(0.15, 0.55, cloud_height));
    if cloud_shape <= 0.001 {
        return 0.0;
    }

    p.x = p.x + globals.time * 12.3;
    var den = max(0.0, cloud_shape - 0.48 * fbm3(p * 0.02));
    if den <= 0.0 {
        return 0.0;
    }

    if detailed {
        p.y = p.y + globals.time * 15.2;
        den = max(0.0, den - 0.14 * fbm3(p * 0.088));
    }
    return large_weather * 0.35 * min(1.0, 5.0 * den);
}

fn numerical_mie_fit(costh: f32) -> f32 {
    let p1 = costh + 0.8194068;
    let exp_values = exp(vec4<f32>(
        -65.0 * costh - 55.0,
        -83.70334 * p1 * p1,
        7.810083 * costh,
        -4.552125e-12 * costh
    ));
    let exp_val_weight = vec4<f32>(9.805233e-06, 0.1388198, 0.002054747, 0.02600563);
    return dot(exp_values, exp_val_weight);
}

fn light_ray(
    p0: vec3<f32>,
    phase_function: f32,
    d_c: f32,
    mu: f32,
    sun_direction: vec3<f32>,
    cloud_height: f32,
) -> f32 {
    let z_max_l = 600.0;
    let step_l = z_max_l / f32(NB_SAMPLE_LIGHT);
    var light_ray_den = 0.0;
    var p = p0 + sun_direction * step_l * hash11(dot(p0, vec3<f32>(12.256, 2.646, 6.356)) + globals.time);
    for (var j = 0; j < NB_SAMPLE_LIGHT; j = j + 1) {
        var h: f32;
        // 光照步进用廉价密度，避免再嵌套两层 weather_map
        light_ray_den = light_ray_den + clouds(p + sun_direction * f32(j) * step_l, &h, false);
    }
    let scatter_amount = mix(0.008, 1.0, smoothstep(0.96, 0.0, mu));
    let beers = exp(-step_l * light_ray_den)
        + 0.5 * scatter_amount * exp(-0.1 * step_l * light_ray_den)
        + scatter_amount * 0.4 * exp(-0.02 * step_l * light_ray_den);
    return beers
        * phase_function
        * mix(
            0.05 + 1.5 * pow(min(1.0, d_c * 8.5), 0.3 + 5.5 * cloud_height),
            1.0,
            clamp(light_ray_den * 0.4, 0.0, 1.0)
        );
}

fn sky_ray(org: vec3<f32>, dir: vec3<f32>, sun_direction: vec3<f32>) -> vec3<f32> {
    let atm_start = EARTH_RADIUS + CLOUD_START;
    let atm_end = atm_start + CLOUD_HEIGHT;
    var color = vec3<f32>(0.0);
    let dist_to_atm_start = intersect_sphere(org, dir, vec3<f32>(0.0, -EARTH_RADIUS, 0.0), atm_start);
    let dist_to_atm_end = intersect_sphere(org, dir, vec3<f32>(0.0, -EARTH_RADIUS, 0.0), atm_end);

    if dist_to_atm_start < 0.0 || dist_to_atm_end < 0.0 {
        let mu = dot(sun_direction, dir);
        return 6.0 * mix(vec3<f32>(0.2, 0.52, 1.0), vec3<f32>(0.8, 0.95, 1.0), pow(0.5 + 0.5 * mu, 15.0))
            + mix(vec3<f32>(3.5), vec3<f32>(0.0), min(1.0, 2.3 * dir.y));
    }

    var p = org + dist_to_atm_start * dir;
    let step_s = (dist_to_atm_end - dist_to_atm_start) / f32(NB_SAMPLE);
    var t = 1.0;
    let mu = dot(sun_direction, dir);
    let phase_function = numerical_mie_fit(mu);
    p = p + dir * step_s * hash11(dot(dir, vec3<f32>(12.256, 2.646, 6.356)) + globals.time);

    if dir.y > 0.015 {
        for (var i = 0; i < NB_SAMPLE; i = i + 1) {
            var cloud_height: f32;
            let density = clouds(p, &cloud_height, true);
            if density > 0.0 {
                let intensity = light_ray(p, phase_function, density, mu, sun_direction, cloud_height);
                let ambient = (0.5 + 0.6 * cloud_height) * vec3<f32>(0.2, 0.5, 1.0) * 6.5
                    + vec3<f32>(0.8) * max(0.0, 1.0 - 2.0 * cloud_height);
                var radiance = ambient + SUN_POWER * intensity;
                radiance = radiance * density;
                color = color + t * (radiance - radiance * exp(-density * step_s)) / density;
                t = t * exp(-density * step_s);
                if t <= 0.05 {
                    break;
                }
            }
            p = p + dir * step_s;
        }
    }

    let p_c = org
        + intersect_sphere(org, dir, vec3<f32>(0.0, -EARTH_RADIUS, 0.0), atm_end + 1000.0) * dir;
    color = color + t * vec3<f32>(3.5) * max(0.0, fbm3(vec3<f32>(1.0, 1.0, 1.8) * p_c * 0.0008) - 0.32);

    var background = 6.0
        * mix(vec3<f32>(0.2, 0.52, 1.0), vec3<f32>(0.8, 0.95, 1.0), pow(0.5 + 0.5 * mu, 15.0))
        + mix(vec3<f32>(3.5), vec3<f32>(0.0), min(1.0, 2.3 * dir.y));
    background = background + t * vec3<f32>(1e4 * smoothstep(0.9998, 1.0, mu));
    color = color + background * t;
    return color;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let dir = normalize(in.world_position.xyz - view.world_position);
    let sun = normalize(sky.sun_dir);
    let org = vec3<f32>(0.0, 3.0, 0.0);

    var color: vec3<f32>;
    if dir.y < -0.02 {
        let mu = dot(sun, dir);
        color = 6.0 * mix(vec3<f32>(0.2, 0.52, 1.0), vec3<f32>(0.8, 0.95, 1.0), pow(0.5 + 0.5 * mu, 15.0))
            + mix(vec3<f32>(3.5), vec3<f32>(1.2), min(1.0, 2.3 * (-dir.y)));
    } else {
        color = sky_ray(org, dir, sun);
    }

    return vec4<f32>(color * sky.exposure, 1.0);
}
