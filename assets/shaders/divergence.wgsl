#import bevy_fluid::coordinate::{left, right, bottom, top};

@group(0) @binding(2) var u1: texture_storage_2d<r32float, read_write>;
@group(0) @binding(3) var v1: texture_storage_2d<r32float, read_write>;

@group(1) @binding(2) var div: texture_storage_2d<r32float, read_write>;
@group(1) @binding(3) var grid_label: texture_storage_2d<r32uint, read_write>;

@compute @workgroup_size(8, 8, 1)
fn divergence(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let label = textureLoad(grid_label, x).r;
    if (label == 2) {
        textureStore(div, x, vec4<f32>(0.0, 0.0, 0.0, 0.0));
        return;
    }
    
    var rhs: f32 = 0.0;
    let dim_u = vec2<i32>(textureDimensions(u1));
    let grid_iplus_j = textureLoad(grid_label, right(x)).r;
    if (x.x < dim_u.x - 1 && grid_iplus_j == 2) {
        let u_solid = textureLoad(u1, right(x)).r;
        rhs += textureLoad(u1, right(x)).r - u_solid;
    }
    let grid_iminus_j = textureLoad(grid_label, left(x)).r;
    if (0 < x.x && grid_iminus_j == 2) {
        let u_solid = textureLoad(u1, left(x)).r;
        rhs -= textureLoad(u1, x).r - u_solid;
    }

    let dim_v = vec2<i32>(textureDimensions(v1));
    let grid_i_jplus = textureLoad(grid_label, top(x)).r;
    if (x.y < dim_v.y - 1 && grid_i_jplus == 2) {
        let v_solid = textureLoad(v1, top(x)).r;
        rhs += textureLoad(v1, top(x)).r - v_solid;
    }
    let grid_i_jminus = textureLoad(grid_label, bottom(x)).r;
    if (0 < x.y && grid_i_jminus == 2) {
        let v_solid = textureLoad(v1, bottom(x)).r;
        rhs -= textureLoad(v1, x).r - v_solid;
    }

    let u0 = textureLoad(u1, x).r;
    let u1 = textureLoad(u1, right(x)).r;
    let v0 = textureLoad(v1, x).r;
    let v1 = textureLoad(v1, top(x)).r;
    let result = vec4<f32>(u1 - u0 + v1 - v0 - rhs, 0.0, 0.0, 0.0);
    textureStore(div, x, result);
}