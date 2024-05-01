@group(0) @binding(0) var velocity_texture: texture_storage_2d<rg32float, read_write>;

@compute @workgroup_size(8, 8, 1)
fn init(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
    @builtin(num_workgroups) num_workgroups: vec3<u32>,
) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let speed = gausian_2d(256.0 - f32(invocation_id.x), 256.0 - f32(invocation_id.y), 50.0);
    let velocity = vec2<f32>(speed, 0);
    textureStore(velocity_texture, location, vec4<f32>(velocity, 0.0, 0.0));
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let backtraced_location: vec2<f32> = runge_kutta(location);
    let backtraced_texture: vec2<f32> = interpolate_texture(backtraced_location);
    
    textureStore(velocity_texture, location, vec4<f32>(backtraced_texture, 0.0, 0.0));
}

fn gausian_2d(x: f32, y: f32, sigma: f32) -> f32 {
    let b = -1.0 / (2.0 * sigma * sigma);
    return exp(b * (x * x + y * y));
}

fn runge_kutta(location: vec2<i32>) -> vec2<f32> {
    let dt = 1.0;
    let x_mid = vec2<f32>(location) + vec2<f32>(0.5 * dt) * vec2<f32>(textureLoad(velocity_texture, location).rg);

    return vec2<f32>(location) + dt * interpolate_texture(x_mid);
}

fn interpolate_texture(location: vec2<f32>) -> vec2<f32> {
    let floor_location = vec2<i32>(i32(location.x), i32(location.y));
    let fract_location = location - vec2<f32>(f32(floor_location.x), f32(floor_location.y));

    let top_left = textureLoad(velocity_texture, floor_location).rg;
    let top_right = textureLoad(velocity_texture, floor_location + vec2<i32>(1, 0)).rg;
    let bottom_left = textureLoad(velocity_texture, floor_location + vec2<i32>(0, 1)).rg;
    let bottom_right = textureLoad(velocity_texture, floor_location + vec2<i32>(1, 1)).rg;

    let top = mix(top_left, top_right, fract_location.x);
    let bottom = mix(bottom_left, bottom_right, fract_location.x);

    return mix(top, bottom, fract_location.y);
}