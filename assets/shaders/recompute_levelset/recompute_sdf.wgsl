@group(0) @binding(0) var seeds: texture_storage_2d<rg32float, read_write>;
@group(1) @binding(0) var levelset: texture_storage_2d<r32float, read_write>;

@compute
@workgroup_size(8, 8, 1)
fn sdf(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let sdf = distance(vec2<f32>(textureLoad(seeds, x).xy), vec2<f32>(x));
    let level = textureLoad(levelset, x).r;
    var levelset_sign = 1.0;
    if (level < 0.0) {
        levelset_sign = -1.0;
    }
    textureStore(levelset, x, vec4<f32>(levelset_sign * sdf, 0.0, 0.0, 0.0));
}