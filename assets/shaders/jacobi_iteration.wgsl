#import bevy_fluid::fluid_uniform::SimulationUniform;
#import bevy_fluid::coordinate::{left, right, bottom, top};

@group(0) @binding(0) var div: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var p_in: texture_storage_2d<r32float, read_write>;
@group(0) @binding(2) var p_out: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var<uniform> constants: SimulationUniform;

@group(2) @binding(0) var grid_label: texture_storage_2d<r32uint, read_write>;

@compute @workgroup_size(8, 8, 1)
fn jacobi_iteration(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let label = textureLoad(grid_label, x).r;
    if (label == 2) {
        textureStore(p_out, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
        return;
    }
    
    let coef = 4.0 
        - is_solid(grid_label, left(x))
        - is_solid(grid_label, right(x))
        - is_solid(grid_label, bottom(x))
        - is_solid(grid_label, top(x));

    if (coef == 0.0) {
        textureStore(p_out, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
        return;
    } else {
        let p_left = pij(p_in, grid_label, left(x));
        let p_right = pij(p_in, grid_label, right(x));
        let p_bottom = pij(p_in, grid_label, bottom(x));
        let p_top = pij(p_in, grid_label, top(x));
        let div_ij = textureLoad(div, x).r;

        let factor = constants.dx * constants.rho / constants.dt;
        let p = (1.0 / coef) * (p_left + p_right + p_bottom + p_top - factor * div_ij);
        textureStore(p_out, x, vec4<f32>(p, 0.0, 0.0, 0.0));
    }
}

@compute @workgroup_size(8, 8, 1)
fn swap_buffers(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let p = textureLoad(p_out, x).r;
    textureStore(p_in, x, vec4<f32>(p, 0.0, 0.0, 0.0));
}

fn pij(p: texture_storage_2d<r32float, read_write>, label: texture_storage_2d<r32uint, read_write>, x: vec2<i32>) -> f32 {
    return textureLoad(p, x).r * is_fluid(label, x);
}

fn is_solid(label: texture_storage_2d<r32uint, read_write>, x: vec2<i32>) -> f32 {
    if (textureLoad(label, x).r == 2) {
        return 1.0;
    } else {
        return 0.0;
    }
}

fn is_fluid(label: texture_storage_2d<r32uint, read_write>, x: vec2<i32>) -> f32 {
    if (textureLoad(label, x).r == 1) {
        return 1.0;
    } else {
        return 0.0;
    }
}