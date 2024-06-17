#import bevy_fluid::fluid_uniform::SimulationUniform;

@group(0) @binding(0) var u_in: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v_in: texture_storage_2d<r32float, read_write>;
@group(0) @binding(2) var u_out: texture_storage_2d<r32float, read_write>;
@group(0) @binding(3) var v_out: texture_storage_2d<r32float, read_write>;
@group(0) @binding(4) var pressure: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

@compute @workgroup_size(1, 64, 1)
fn solve_pressure(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let factor = constants.dt / (constants.dx * constants.rho);

    let x_u = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let p1_u = textureLoad(pressure, x_u).r;
    let p0_u = textureLoad(pressure, x_u - vec2<i32>(1, 0)).r;
    let u = textureLoad(u_in, x_u);
    let du = vec4<f32>(factor * (p1_u - p0_u), 0.0, 0.0, 0.0);

    let x_v = vec2<i32>(x_u.y, x_u.x);
    let p1_v = textureLoad(pressure, x_v).r;
    let p0_v = textureLoad(pressure, x_v - vec2<i32>(0, 1)).r;
    let v = textureLoad(v_in, x_v);
    let dv = vec4<f32>(factor * (p1_v - p0_v), 0.0, 0.0, 0.0);

    textureStore(u_out, x_u, u - du);
    textureStore(v_out, x_v, v - dv);
}