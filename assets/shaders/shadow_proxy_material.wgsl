// 阴影代理：主视角丢弃像素；Metal 要求 discard 后仍有 return，且勿留未使用入参
#import bevy_pbr::forward_io::VertexOutput

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let _keep = in.position.w;
    discard;
    return vec4<f32>(0.0, 0.0, 0.0, 0.0) * _keep;
}
