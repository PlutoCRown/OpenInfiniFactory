// MC 风格传送门：半透明紫红，自中心向外螺旋；params.w 为闪烁强度
#import bevy_pbr::{
    mesh_view_bindings::globals,
    forward_io::VertexOutput,
}

struct PortalUniform {
    base_color: vec4<f32>,
    flow_color: vec4<f32>,
    /// x: 螺旋转速；y: 径向条纹密度；z: 臂数；w: 闪烁 0～1
    params: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material: PortalUniform;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = globals.time;
    let speed = material.params.x;
    let density = max(material.params.y, 0.001);
    let arms = max(material.params.z, 1.0);
    let flash = clamp(material.params.w, 0.0, 1.0);

    // 面 UV 中心 → 极坐标螺旋（疏一些）
    let p = in.uv - vec2<f32>(0.5, 0.5);
    let radius = length(p) * 2.0;
    let angle = atan2(p.y, p.x);
    let tau = 6.28318530718;
    let spiral = fract(radius * density - angle * arms / tau - t * speed);
    let band = smoothstep(0.22, 0.0, abs(spiral - 0.5));
    let wash = 0.45 + 0.55 * (1.0 - smoothstep(0.0, 1.2, radius));

    var rgb = mix(material.base_color.rgb, material.flow_color.rgb, band * wash);
    rgb = rgb + material.flow_color.rgb * flash * 2.4;
    rgb = rgb + vec3<f32>(0.35, 0.12, 0.55) * flash;

    let alpha = material.base_color.a * (0.55 + 0.30 * band + 0.15 * wash) * (0.88 + 0.12 * flash);
    return vec4<f32>(rgb, alpha);
}
