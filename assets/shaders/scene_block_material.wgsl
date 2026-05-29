#import bevy_pbr::forward_io::VertexOutput

struct SceneBlockMaterial {
    base_color: vec4<f32>,
    accent_color: vec4<f32>,
    texture_kind: u32,
};

@group(2) @binding(0)
var<uniform> material: SceneBlockMaterial;

fn hash3(p: vec3<f32>) -> f32 {
    let q = fract(p * vec3<f32>(0.1031, 0.11369, 0.13787));
    let d = dot(q, q.yzx + vec3<f32>(19.19));
    return fract((q.x + q.y) * q.z + d);
}

fn value_noise(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let n000 = hash3(i + vec3<f32>(0.0, 0.0, 0.0));
    let n100 = hash3(i + vec3<f32>(1.0, 0.0, 0.0));
    let n010 = hash3(i + vec3<f32>(0.0, 1.0, 0.0));
    let n110 = hash3(i + vec3<f32>(1.0, 1.0, 0.0));
    let n001 = hash3(i + vec3<f32>(0.0, 0.0, 1.0));
    let n101 = hash3(i + vec3<f32>(1.0, 0.0, 1.0));
    let n011 = hash3(i + vec3<f32>(0.0, 1.0, 1.0));
    let n111 = hash3(i + vec3<f32>(1.0, 1.0, 1.0));

    let nx00 = mix(n000, n100, u.x);
    let nx10 = mix(n010, n110, u.x);
    let nx01 = mix(n001, n101, u.x);
    let nx11 = mix(n011, n111, u.x);
    let nxy0 = mix(nx00, nx10, u.y);
    let nxy1 = mix(nx01, nx11, u.y);
    return mix(nxy0, nxy1, u.z);
}

fn fbm(p: vec3<f32>) -> f32 {
    var value = 0.0;
    var amp = 0.5;
    var freq = 1.0;
    for (var i = 0; i < 4; i = i + 1) {
        value = value + value_noise(p * freq) * amp;
        freq = freq * 2.17;
        amp = amp * 0.5;
    }
    return value;
}

fn scene_pattern(world_pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    let p = world_pos * 2.65;
    var n = fbm(p);

    if (material.texture_kind == 0u) {
        n = n + smoothstep(0.72, 0.93, fbm(world_pos * vec3<f32>(6.8, 1.6, 6.8))) * 0.22;
    } else if (material.texture_kind == 1u) {
        let strata = sin((world_pos.y * 5.8 + world_pos.x * 0.72 + world_pos.z * 0.44) * 3.14159);
        n = n * 0.72 + strata * 0.09 + fbm(world_pos * 8.4) * 0.20;
    } else if (material.texture_kind == 2u) {
        n = n * 0.72 + fbm(world_pos * vec3<f32>(5.2, 2.4, 5.2) + vec3<f32>(8.1)) * 0.30;
    } else if (material.texture_kind == 3u) {
        let boards = smoothstep(0.04, 0.09, abs(fract(world_pos.x * 1.0) - 0.5));
        let grain = fbm(world_pos * vec3<f32>(9.6, 1.3, 2.2));
        n = grain * 0.55 + boards * 0.18;
    } else {
        n = fbm(world_pos * 5.2 + normal * 2.0);
    }

    return clamp(n, 0.0, 1.0);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let world_pos = in.world_position.xyz;
    let normal = normalize(in.world_normal);
    let n = scene_pattern(world_pos, normal);
    let face_light = 0.78 + max(dot(normal, normalize(vec3<f32>(0.35, 0.8, 0.25))), 0.0) * 0.22;
    let color = mix(material.base_color.rgb, material.accent_color.rgb, n) * face_light;
    return vec4<f32>(color, material.base_color.a);
}
