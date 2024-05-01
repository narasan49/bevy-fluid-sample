@group(0) @binding(0) var velocity_texture: texture_storage_2d<rg32float, read_write>;

@compute @workgroup_size(8, 8, 1)
fn init(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
    @builtin(num_workgroups) num_workgroups: vec3<u32>,
) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    // var backtraced_location = runge_kutta(mesh.uv);
    // return textureSample(velocity_texture, velocity_sampler, backtraced_location).rg;
    let velocity = vec2<f32>(1.0, 0.5);
    textureStore(velocity_texture, location, vec4<f32>(velocity, 2.0, 1.0));
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {

}

// fn runge_kutta(tex_coords: vec2<f32>) -> vec2<f32> {
//     var x_mid = tex_coords + 0.5 * dt * textureSample(velocity_texture, velocity_sampler, tex_coords).rg;
//     return tex_coords + dt * textureSample(velocity_texture, velocity_sampler, x_mid).rg;
// }