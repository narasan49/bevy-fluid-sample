#import bevy_sprite::mesh2d_vertex_output::VertexOutput;

@group(2) @binding(0) var<uniform> scale: vec3<f32>;
@group(2) @binding(1) var velocity_texture: texture_2d<f32>;
@group(2) @binding(2) var velocity_sampler: sampler;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    var v = textureSample(velocity_texture, velocity_sampler, mesh.uv).r;
    var negative = step(v, 0.0);
    return vec4<f32>(- scale * negative * v, 1.0);
}
