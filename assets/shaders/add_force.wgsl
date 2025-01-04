#import bevy_fluid::fluid_uniform::SimulationUniform;

@group(0) @binding(2) var u1: texture_storage_2d<r32float, read_write>;
@group(0) @binding(3) var v1: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

@group(2) @binding(0) var<storage, read> force: array<vec2<f32>>;
@group(2) @binding(1) var<storage, read> position: array<vec2<f32>>;

@compute
@workgroup_size(1, 64, 1)
fn add_force(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x_u = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let x_v = vec2<i32>(x_u.y, x_u.x);
    var n = arrayLength(&force);
    var net_force = vec2<f32>(0.0, 0.0);
    net_force.y = constants.gravity.y;
    net_force.x = constants.gravity.x;

    loop {
        if (n == 0) {
            break;
        }
        n = n - 1u;
        let f = force[n];
        let p = position[n];
        let force_u = f.x * gaussian_2d(vec2<f32>(x_u), p, 10.0);
        let force_v = f.y * gaussian_2d(vec2<f32>(x_v), p, 10.0);
        net_force = net_force + vec2<f32>(force_u, force_v);
    }

    let u_val = textureLoad(u1, x_u).r;
    let v_val = textureLoad(v1, x_v).r;
    textureStore(u1, x_u, vec4<f32>(u_val + net_force.x * constants.dt, 0.0, 0.0, 0.0));
    textureStore(v1, x_v, vec4<f32>(v_val + net_force.y * constants.dt, 0.0, 0.0, 0.0));
}

fn gaussian_2d(x: vec2<f32>, x0: vec2<f32>, sigma: f32) -> f32 {
    let b = -1.0 / (2.0 * sigma * sigma);
    return exp(b * dot(x - x0, x - x0));
}