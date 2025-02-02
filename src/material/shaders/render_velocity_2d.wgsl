#import bevy_sprite::mesh2d_vertex_output::VertexOutput;

@group(2) @binding(0) var<uniform> u_range: vec2<f32>;
@group(2) @binding(1) var<uniform> v_range: vec2<f32>;
@group(2) @binding(2) var u_tex: texture_2d<f32>;
@group(2) @binding(3) var u_sampler: sampler;
@group(2) @binding(4) var v_tex: texture_2d<f32>;
@group(2) @binding(5) var v_sampler: sampler;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let u = textureSample(u_tex, u_sampler, mesh.uv).r;
    let v = textureSample(v_tex, v_sampler, mesh.uv).r;
    let scaled_u = (u - u_range.x) / (u_range.y - u_range.x);
    let scaled_v = (v - v_range.x) / (v_range.y - v_range.x);
    return vec4<f32>(scaled_u, scaled_v, 0.5, 1.0);
}