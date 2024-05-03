#import bevy_pbr::forward_io::VertexOutput;

@group(2) @binding(0) var<uniform> base_color: vec4<f32>;
@group(2) @binding(1) var velocity_texture: texture_2d<f32>;
@group(2) @binding(2) var velocity_sampler: sampler;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    // var backtraced_location = runge_kutta(mesh.uv);
    var v = textureSample(velocity_texture, velocity_sampler, mesh.uv).rg;
    
    return vec4<f32>(v, 0.0, 1.0);
}