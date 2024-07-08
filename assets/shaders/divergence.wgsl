@group(0) @binding(0) var u_in: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v_in: texture_storage_2d<r32float, read_write>;
@group(0) @binding(2) var div: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var grid_label: texture_storage_2d<r32uint, read_write>;

@compute @workgroup_size(8, 8, 1)
fn divergence(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let u_solid = 0.0;
    let v_solid = 0.0;
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    
    var rhs: f32 = 0.0;
    let grid_iplus_j = textureLoad(grid_label, x + vec2<i32>(1, 0)).r;
    if (grid_iplus_j == 2) {
        rhs -= textureLoad(u_in, x + vec2<i32>(1, 0)).r - u_solid;
    }
    let grid_iminus_j = textureLoad(grid_label, x + vec2<i32>(-1, 0)).r;
    if (grid_iminus_j == 2) {
        rhs -= textureLoad(u_in, x).r - u_solid;
    }
    let grid_i_jplus = textureLoad(grid_label, x + vec2<i32>(0, 1)).r;
    if (grid_i_jplus == 2) {
        rhs -= textureLoad(v_in, x + vec2<i32>(0, 1)).r - v_solid;
    }
    let grid_i_jminus = textureLoad(grid_label, x + vec2<i32>(0, -1)).r;
    if (grid_i_jminus == 2) {
        rhs -= textureLoad(v_in, x).r - v_solid;
    }

    let u0 = textureLoad(u_in, x).r;
    let u1 = textureLoad(u_in, x + vec2<i32>(1, 0)).r;
    let v0 = textureLoad(v_in, x).r;
    let v1 = textureLoad(v_in, x + vec2<i32>(0, 1)).r;
    let result = vec4<f32>(u1 - u0 + v1 - v0 + rhs, 0.0, 0.0, 0.0);
    textureStore(div, x, result);
}