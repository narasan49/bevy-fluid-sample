@group(0) @binding(0) var velocity_texture: texture_storage_2d<rg32float, read_write>;
@group(0) @binding(1) var intermediate_velocity: texture_storage_2d<rg32float, read_write>;
@group(0) @binding(2) var pressure: texture_storage_2d<r32float, read_write>;
@group(0) @binding(3) var intermediate_pressure: texture_storage_2d<r32float, read_write>;
@group(0) @binding(4) var boundary_condition: texture_storage_2d<r32float, read_write>;
@group(0) @binding(5) var<uniform> constants: SimulationUniform;

struct SimulationUniform {
    dx: f32,
    dt: f32,
    rho: f32,
}

@compute @workgroup_size(8, 8, 1)
fn init(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
    @builtin(num_workgroups) num_workgroups: vec3<u32>,
) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let speed = 1.0 * gausian_2d(256.0 - f32(invocation_id.x), 256.0 - f32(invocation_id.y), 50.0);
    let velocity = vec2<f32>(speed, 0.0);
    // let u = abs(f32(location.x) - 256.0) * 0.1;
    // let v = abs(f32(location.y) - 256.0) * 0.1;
    // let u = 10.0;
    // let v = 0.0;
    // let velocity = vec2<f32>(u, v);

    textureStore(velocity_texture, location, vec4<f32>(velocity, 0.0, 0.0));
    textureStore(intermediate_velocity, location, vec4<f32>(velocity, 0.0, 0.0));
    textureStore(pressure, location, vec4<f32>(0.0, 0.0, 0.0, 0.0));
    textureStore(intermediate_pressure, location, vec4<f32>(0.0, 0.0, 0.0, 0.0));
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let backtraced_location: vec2<f32> = runge_kutta(velocity_texture, location);
    let backtraced_texture: vec2<f32> = vec2<f32>(u_at(velocity_texture, backtraced_location), v_at(velocity_texture, backtraced_location));
    
    textureStore(intermediate_velocity, location, vec4<f32>(backtraced_texture, 0.0, 0.0));
}

fn gausian_2d(x: f32, y: f32, sigma: f32) -> f32 {
    let b = -1.0 / (2.0 * sigma * sigma);
    return exp(b * (x * x + y * y));
}

fn runge_kutta(
    texture: texture_storage_2d<rg32float, read_write>,
    location: vec2<i32>
) -> vec2<f32> {
    let dt = constants.dt;
    let velocity = vec2<f32>(u_at(texture, vec2<f32>(location)), v_at(texture, vec2<f32>(location)));
    let x_mid = vec2<f32>(location) - vec2<f32>(0.5 * dt) * velocity;
    let velocity_mid = vec2<f32>(u_at(texture, x_mid), v_at(texture, x_mid));

    return vec2<f32>(location) - dt * velocity_mid;
}

fn interpolate2d(
    texture: texture_storage_2d<rg32float, read_write>,
    location: vec2<f32>
) -> vec2<f32> {
    let floor_location = vec2<i32>(i32(location.x), i32(location.y));
    let fract_location = location - vec2<f32>(f32(floor_location.x), f32(floor_location.y));

    let top_left = textureLoad(texture, floor_location).rg;
    let top_right = textureLoad(texture, floor_location + vec2<i32>(1, 0)).rg;
    let bottom_left = textureLoad(texture, floor_location + vec2<i32>(0, 1)).rg;
    let bottom_right = textureLoad(texture, floor_location + vec2<i32>(1, 1)).rg;

    let top = mix(top_left, top_right, fract_location.x);
    let bottom = mix(bottom_left, bottom_right, fract_location.x);

    return mix(top, bottom, fract_location.y);
}

fn u_at(
    velocity: texture_storage_2d<rg32float, read_write>,
    location: vec2<f32>,
) -> f32 {
    let i = i32(round(location.x));
    let j = i32(floor(location.y));
    let fract_i = f32(i) - round(location.x);
    let fract_j = f32(j) - floor(location.y);
    let u00 = textureLoad(velocity, vec2<i32>(i, j)).r;
    let u10 = textureLoad(velocity, vec2<i32>(i + 1, j)).r;
    let u01 = textureLoad(velocity, vec2<i32>(i, j + 1)).r;
    let u11 = textureLoad(velocity, vec2<i32>(i + 1, j + 1)).r;

    return mix(mix(u00, u10, fract_i), mix(u01, u11, fract_i), fract_j);
}

fn v_at(
    velocity: texture_storage_2d<rg32float, read_write>,
    location: vec2<f32>,
) -> f32 {
    let i = i32(floor(location.x));
    let j = i32(round(location.y));
    let fract_i = f32(i) - floor(location.x);
    let fract_j = f32(j) - round(location.y);
    let v00 = textureLoad(velocity, vec2<i32>(i, j)).g;
    let v10 = textureLoad(velocity, vec2<i32>(i + 1, j)).g;
    let v01 = textureLoad(velocity, vec2<i32>(i, j + 1)).g;
    let v11 = textureLoad(velocity, vec2<i32>(i + 1, j + 1)).g;

    return mix(mix(v00, v10, fract_i), mix(v01, v11, fract_i), fract_j);
}

@compute @workgroup_size(8, 8, 1)
fn solve_pressure(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let div = divergence(intermediate_velocity, location);
    let p = jacobi_iteration(pressure, location, div);
    textureStore(intermediate_pressure, location, vec4<f32>(p, 0.0, 0.0, 0.0));
}

fn jacobi_iteration(pressure: texture_storage_2d<r32float, read_write>, location: vec2<i32>, divergence: f32) -> f32 {
    let factor = constants.dx * constants.rho / constants.dt;
    let p_i_minus_j = textureLoad(pressure, location - vec2<i32>(1, 0)).r;
    let p_i_plus_j = textureLoad(pressure, location + vec2<i32>(1, 0)).r;
    let p_i_j_minus = textureLoad(pressure, location - vec2<i32>(0, 1)).r;
    let p_i_j_plus = textureLoad(pressure, location + vec2<i32>(0, 1)).r;

    return 0.25 * (p_i_minus_j + p_i_plus_j + p_i_j_minus + p_i_j_plus - factor * divergence);
}

@compute @workgroup_size(8, 8, 1)
fn update_pressure(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let p = textureLoad(intermediate_pressure, location);
    textureStore(pressure, location, p);
}

// u[i-1/2, j]: textureLoad(velocity_texture, location).r
// u[i+1/2, j]: textureLoad(velocity_texture, location + vec2<i32>(1, 0)).r
fn divergence(velocity_texture: texture_storage_2d<rg32float, read_write>, location: vec2<i32>) -> f32 {
    let uv = textureLoad(velocity_texture, location).rg;
    let u_plus = textureLoad(velocity_texture, location + vec2<i32>(1, 0)).r;
    let v_plus = textureLoad(velocity_texture, location + vec2<i32>(0, 1)).g;

    return (u_plus - uv.x + v_plus - uv.y);
}

// u[i, j] = f * (p[i, j] - p[i-1, j]) is decomposed into three steps focusing on pressure.
// 1. initialize: u[i, j] = 0, v[i, j] = 0
// 2. subtract pressure: u[i+1, j] = - f * p[i, j], v[i, j+1] = - f * p[i, j]
// 3. add pressure: u[i, j] += f * p[i, j], v[i, j] += f * p[i, j]
@compute @workgroup_size(8, 8, 1)
fn solve_velocity_init(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let velocity = textureLoad(intermediate_velocity, location);
    textureStore(velocity_texture, location, velocity);
}

@compute @workgroup_size(8, 8, 1)
fn solve_velocity_sub(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let factor = constants.dt / (constants.dx * constants.rho);
    
    let p_ij = textureLoad(pressure, location).r;
    let delta = p_ij * factor;
    let u_i_plus_j = sample_velocity(velocity_texture, location + vec2<i32>(1, 0));
    let u_i_j_plus = sample_velocity(velocity_texture, location + vec2<i32>(0, 1));
    textureStore(velocity_texture, location + vec2<i32>(1, 0), vec4<f32>(u_i_plus_j + vec2<f32>(delta, 0), 0.0, 0.0));
}

@compute @workgroup_size(8, 8, 1)
fn solve_velocity_sub_v(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let factor = constants.dt / (constants.dx * constants.rho);
    
    let p_ij = textureLoad(pressure, location).r;
    let delta = p_ij * factor;
    let u_i_plus_j = sample_velocity(velocity_texture, location + vec2<i32>(1, 0));
    let u_i_j_plus = sample_velocity(velocity_texture, location + vec2<i32>(0, 1));
    textureStore(velocity_texture, location + vec2<i32>(0, 1), vec4<f32>(u_i_j_plus + vec2<f32>(0.0, delta), 0.0, 0.0));
}

@compute @workgroup_size(8, 8, 1)
fn solve_velocity_add(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let factor = constants.dt / (constants.dx * constants.rho);
    
    let p_ij = textureLoad(pressure, location).r;
    let u_ij = sample_velocity(velocity_texture, location);
    let delta = vec2<f32>(p_ij * factor);

    textureStore(velocity_texture, location, vec4<f32>(u_ij - delta, 0.0, 0.0));
}

fn sample_velocity(texture: texture_storage_2d<rg32float, read_write>, location: vec2<i32>) -> vec2<f32> {
    return textureLoad(texture, location).rg;
}