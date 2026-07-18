// 验收器游玩态：半透明目标材料 + 自下而上亮度扫光
#import bevy_pbr::{
    mesh_view_bindings::globals,
    forward_io::VertexOutput,
}

struct GoalGhostUniform {
    base_color: vec4<f32>,
    sweep_color: vec4<f32>,
    /// x: 世界单位/秒扫速；y: 光带周期（方块格数）；z: 光带半宽（相对周期 0～0.5）
    params: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material: GoalGhostUniform;
@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var base_color_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(base_color_texture, base_color_sampler, in.uv);
    var color = material.base_color * tex;

    // 略向 X/Z 倾斜的扫光轴，相连验收器仍共享连续相位
    let axis = normalize(vec3<f32>(0.28, 1.0, 0.18));
    let period = max(material.params.y, 0.001);
    let speed = material.params.x;
    let half = max(material.params.z, 0.001);
    let along = dot(in.world_position.xyz, axis);
    let phase = fract((along - globals.time * speed) / period);
    let dist = min(phase, 1.0 - phase);
    let sweep = smoothstep(half, 0.0, dist);

    let lit = color.rgb + material.sweep_color.rgb * sweep;
    let alpha = color.a * (0.92 + 0.08 * sweep);
    return vec4<f32>(lit, alpha);
}
