struct SimulationUniform {
    dt: f32,
}

@group(0) @binding(0) var input_velocity: texture_storage_2d<rg32float, read_write>;
@group(0) @binding(1) var output_velocity: texture_storage_2d<rg32float, read_write>;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

@compute @workgroup_size(8, 8, 1)
fn initialize(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let speed = 1.0 * gausian_2d(256.0 - f32(invocation_id.x), 256.0 - f32(invocation_id.y), 100.0);
    let velocity = vec2<f32>(speed);

    textureStore(input_velocity, location, vec4<f32>(velocity, 0.0, 0.0));
    textureStore(output_velocity, location, vec4<f32>(velocity, 0.0, 0.0));
}

@compute @workgroup_size(8, 8, 1)
fn advection(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let backtraced_location: vec2<f32> = runge_kutta(input_velocity, location, constants.dt);
    let backtraced_texture: vec2<f32> = vec2<f32>(u_at(input_velocity, backtraced_location), v_at(input_velocity, backtraced_location));
    
    textureStore(output_velocity, location, vec4<f32>(backtraced_texture, 0.0, 0.0));
}

@compute @workgroup_size(8, 8, 1)
fn swap(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let velocity = textureLoad(output_velocity, location);
    textureStore(input_velocity, location, velocity);
}

fn runge_kutta(
    texture: texture_storage_2d<rg32float, read_write>,
    location: vec2<i32>,
    dt: f32,
) -> vec2<f32> {
    let velocity = vec2<f32>(u_at(texture, vec2<f32>(location)), v_at(texture, vec2<f32>(location)));
    let x_mid = vec2<f32>(location) - vec2<f32>(0.5 * dt) * velocity;
    let velocity_mid = vec2<f32>(u_at(texture, x_mid), v_at(texture, x_mid));

    return vec2<f32>(location) - dt * velocity_mid;
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

fn gausian_2d(x: f32, y: f32, sigma: f32) -> f32 {
    let b = -1.0 / (2.0 * sigma * sigma);
    return exp(b * (x * x + y * y));
}