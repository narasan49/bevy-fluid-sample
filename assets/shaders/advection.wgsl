#import bevy_fluid::fluid_uniform::SimulationUniform;

@group(0) @binding(0) var u_in: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var u_out: texture_storage_2d<r32float, read_write>;
@group(0) @binding(2) var v_in: texture_storage_2d<r32float, read_write>;
@group(0) @binding(3) var v_out: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

@group(2) @binding(0) var grid_label: texture_storage_2d<r32uint, read_write>;

// ToDo: Move to a separate file
@compute @workgroup_size(1, 64, 1)
fn initialize(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x_u = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let x_v = vec2<i32>(x_u.y, x_u.x);
    let speed = 10.0 * gausian_2d(256.0 - f32(invocation_id.x), 256.0 - f32(invocation_id.y), 100.0);

    textureStore(u_in, x_u, vec4<f32>(speed, 0.0, 0.0, 0.0));
    textureStore(u_out, x_u, vec4<f32>(speed, 0.0, 0.0, 0.0));
    textureStore(v_in, x_v, vec4<f32>(speed, 0.0, 0.0, 0.0));
    textureStore(v_out, x_v, vec4<f32>(speed, 0.0, 0.0, 0.0));

    // Initialize grid_label
    let circle = u32(step(length(vec2<f32>(f32(invocation_id.x) - 128.0, f32(invocation_id.y) - 128.0)), 50.0) + 1);
    textureStore(grid_label, x_u, vec4<u32>(circle, 0, 0, 0));
}

@compute @workgroup_size(1, 64, 1)
fn advection(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x_u = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let x_v = vec2<i32>(x_u.y, x_u.x);

    let label_u = textureLoad(grid_label, x_u - vec2<i32>(0, 1)).r;
    let label_uplus = textureLoad(grid_label, x_u).r;
    if (label_u == 2 || label_uplus == 2) {
        let u_solid = 0.0;
        textureStore(u_out, x_u, vec4<f32>(u_solid, 0.0, 0.0, 0.0));
    } else if (label_u == 0 || label_uplus == 0) {
        textureStore(u_out, x_u, vec4<f32>(0.0, 0.0, 0.0, 0.0));
    } else {
        let backtraced_x_u: vec2<f32> = runge_kutta(u_in, v_in, x_u, constants.dt);
        let backtraced_u: f32 = u_at(u_in, backtraced_x_u);
        textureStore(u_out, x_u, vec4<f32>(backtraced_u, 0.0, 0.0, 0.0));
    }

    let label_v = textureLoad(grid_label, x_v - vec2<i32>(0, 1)).r;
    let label_vplus = textureLoad(grid_label, x_v).r;
    if (label_v == 2 || label_vplus == 2) {
        let v_solid = 0.0;
        textureStore(v_out, x_v, vec4<f32>(v_solid, 0.0, 0.0, 0.0));
    } else if (label_v == 0 || label_vplus == 0) {
        textureStore(v_out, x_v, vec4<f32>(0.0, 0.0, 0.0, 0.0));
    } else {
        let backtraced_x_v: vec2<f32> = runge_kutta(u_in, v_in, x_v, constants.dt);
        let backtraced_v: f32 = v_at(v_in, backtraced_x_v);
        textureStore(v_out, x_v, vec4<f32>(backtraced_v, 0.0, 0.0, 0.0));
    }
}

// ToDo: Move to a separate file
@compute @workgroup_size(1, 64, 1)
fn swap(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x_u = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let u_tmp = textureLoad(u_out, x_u);
    textureStore(u_in, x_u, u_tmp);

    let x_v = vec2<i32>(x_u.y, x_u.x);
    let v_tmp = textureLoad(v_out, x_v);
    textureStore(v_in, x_v, v_tmp);
}

fn runge_kutta(
    u: texture_storage_2d<r32float, read_write>,
    v: texture_storage_2d<r32float, read_write>,
    x: vec2<i32>,
    dt: f32,
) -> vec2<f32> {
    let velocity = vec2<f32>(u_at(u, vec2<f32>(x)), v_at(v, vec2<f32>(x)));
    let x_mid = vec2<f32>(x) - vec2<f32>(0.5 * dt) * velocity;
    let velocity_mid = vec2<f32>(u_at(u, x_mid), v_at(v, x_mid));

    return vec2<f32>(x) - dt * velocity_mid;
}

fn u_at(
    u: texture_storage_2d<r32float, read_write>,
    x: vec2<f32>,
) -> f32 {
    let i = i32(round(x.x));
    let j = i32(floor(x.y));
    let fract_i = f32(i) - round(x.x);
    let fract_j = f32(j) - floor(x.y);
    let u00 = textureLoad(u, vec2<i32>(i, j)).r;
    let u10 = textureLoad(u, vec2<i32>(i + 1, j)).r;
    let u01 = textureLoad(u, vec2<i32>(i, j + 1)).r;
    let u11 = textureLoad(u, vec2<i32>(i + 1, j + 1)).r;

    return mix(mix(u00, u10, fract_i), mix(u01, u11, fract_i), fract_j);
}

fn v_at(
    v: texture_storage_2d<r32float, read_write>,
    x: vec2<f32>,
) -> f32 {
    let i = i32(floor(x.x));
    let j = i32(round(x.y));
    let fract_i = f32(i) - floor(x.x);
    let fract_j = f32(j) - round(x.y);
    let v00 = textureLoad(v, vec2<i32>(i, j)).r;
    let v10 = textureLoad(v, vec2<i32>(i + 1, j)).r;
    let v01 = textureLoad(v, vec2<i32>(i, j + 1)).r;
    let v11 = textureLoad(v, vec2<i32>(i + 1, j + 1)).r;

    return mix(mix(v00, v10, fract_i), mix(v01, v11, fract_i), fract_j);
}

fn gausian_2d(x: f32, y: f32, sigma: f32) -> f32 {
    let b = -1.0 / (2.0 * sigma * sigma);
    return exp(b * (x * x + y * y));
}