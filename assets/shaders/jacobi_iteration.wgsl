#import bevy_fluid::fluid_uniform::SimulationUniform;

@group(0) @binding(0) var div: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var p_in: texture_storage_2d<r32float, read_write>;
@group(0) @binding(2) var p_out: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

@compute @workgroup_size(8, 8, 1)
fn jacobi_iteration(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let factor = constants.dx * constants.rho / constants.dt;

    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let p_i_minus_j = textureLoad(p_in, x - vec2<i32>(1, 0)).r;
    let p_i_plus_j = textureLoad(p_in, x + vec2<i32>(1, 0)).r;
    let p_i_j_minus = textureLoad(p_in, x - vec2<i32>(0, 1)).r;
    let p_i_j_plus = textureLoad(p_in, x + vec2<i32>(0, 1)).r;
    let div_ij = textureLoad(div, x).r;

    let p = 0.25 * (p_i_minus_j + p_i_plus_j + p_i_j_minus + p_i_j_plus - factor * div_ij);
    textureStore(p_out, x, vec4<f32>(p, 0.0, 0.0, 0.0));
}

@compute @workgroup_size(8, 8, 1)
fn swap_buffers(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let p = textureLoad(p_out, x).r;
    textureStore(p_in, x, vec4<f32>(p, 0.0, 0.0, 0.0));
}