@group(0) @binding(0) var u_in: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v_in: texture_storage_2d<r32float, read_write>;
@group(0) @binding(2) var div: texture_storage_2d<r32float, read_write>;

@compute @workgroup_size(8, 8, 1)
fn divergence(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let u0 = textureLoad(u_in, x).r;
    let u1 = textureLoad(u_in, x + vec2<i32>(1, 0)).r;
    let v0 = textureLoad(v_in, x).r;
    let v1 = textureLoad(v_in, x + vec2<i32>(0, 1)).r;
    let result = vec4<f32>(u1 - u0 + v1 - v0, 0.0, 0.0, 0.0);
    textureStore(div, x, result);
}