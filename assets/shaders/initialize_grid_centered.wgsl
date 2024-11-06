@group(0) @binding(0) var levelset: texture_storage_2d<r32float, read_write>;

@compute
@workgroup_size(8, 8, 1)
fn initialize(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let zero_contour_height  = 100.0;
    let value = zero_contour_height - f32(invocation_id.y);
    textureStore(levelset, x, vec4<f32>(value, 0.0, 0.0, 0.0));
}