@group(0) @binding(3) var grid_label: texture_storage_2d<r32uint, read_write>;
@group(0) @binding(4) var u_solid: texture_storage_2d<r32float, read_write>;
@group(0) @binding(5) var v_solid: texture_storage_2d<r32float, read_write>;

@compute
@workgroup_size(8, 8, 1)
fn initialize_grid_center(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = vec2<i32>(i32(global_id.x), i32(global_id.y));
    textureStore(grid_label, x, vec4<u32>(1, 0, 0, 0));
    textureStore(u_solid, x, vec4<f32>(0, 0, 0, 0));
    textureStore(v_solid, x, vec4<f32>(0, 0, 0, 0));
}