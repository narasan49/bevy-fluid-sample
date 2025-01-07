#import bevy_fluid::fluid_uniform::SimulationUniform;

@group(0) @binding(0) var u0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(2) var u1: texture_storage_2d<r32float, read_write>;
@group(0) @binding(3) var v1: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

@group(2) @binding(1) var p1: texture_storage_2d<r32float, read_write>;

@group(3) @binding(1) var grid_label: texture_storage_2d<r32uint, read_write>;

@compute
@workgroup_size(1, 64, 1)
fn solve_velocity(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let factor = constants.dt / (constants.dx * constants.rho);

    let x_u = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let grid_label_u0 = textureLoad(grid_label, x_u - vec2<i32>(1, 0)).r;
    let grid_label_u1 = textureLoad(grid_label, x_u).r;
    if (grid_label_u0 == 2) {
        let u_solid = textureLoad(u1, x_u - vec2<i32>(1, 0)).r;
        textureStore(u0, x_u, vec4<f32>(u_solid, 0.0, 0.0, 0.0));
    } else if (grid_label_u1 == 2) {
        let u_solid = textureLoad(u1, x_u).r;
        textureStore(u0, x_u, vec4<f32>(u_solid, 0.0, 0.0, 0.0));
    } else {
        let p1_u = textureLoad(p1, x_u).r;
        var p0_u = 0.0;
        if x_u.x != 0 {
            p0_u = textureLoad(p1, x_u - vec2<i32>(1, 0)).r;
        }
        let u = textureLoad(u1, x_u);
        let du = vec4<f32>(factor * (p1_u - p0_u), 0.0, 0.0, 0.0);
        textureStore(u0, x_u, u - du);
    }

    let x_v = vec2<i32>(x_u.y, x_u.x);

    let grid_label_v0 = textureLoad(grid_label, x_v - vec2<i32>(0, 1)).r;
    let grid_label_v1 = textureLoad(grid_label, x_v).r;
    if (grid_label_v0 == 2) {
        let v_solid = textureLoad(v1, x_v - vec2<i32>(0, 1)).r;
        textureStore(v0, x_v, vec4<f32>(v_solid, 0.0, 0.0, 0.0));
    } else if (grid_label_v1 == 2) {
        let v_solid = textureLoad(v1, x_v).r;
        textureStore(v0, x_v, vec4<f32>(v_solid, 0.0, 0.0, 0.0));
    } else {
        let p1_v = textureLoad(p1, x_v).r;
        var p0_v = 0.0;
        if x_v.y != 0 {
            p0_v = textureLoad(p1, x_v - vec2<i32>(0, 1)).r;
        }
        let v = textureLoad(v1, x_v);
        let dv = vec4<f32>(factor * (p1_v - p0_v), 0.0, 0.0, 0.0);
        textureStore(v0, x_v, v - dv);
    }
}

fn is_solid(label: texture_storage_2d<r32uint, read_write>, x: vec2<i32>) -> f32 {
    if (textureLoad(label, x).r == 2) {
        return 1.0;
    } else {
        return 0.0;
    }
}