@group(0) @binding(0) var levelset: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var seeds: texture_storage_2d<rg32float, read_write>;

@compute
@workgroup_size(8, 8, 1)
fn initialize(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let x = vec2<i32>(i32(global_id.x), i32(global_id.y));
    let level = textureLoad(levelset, x).r;
    var min_distance = 10.0;
    var min_distance_seed = vec2<f32>(-1.0, -1.0);

    for (var i: i32 = -1; i <= 1; i++) {
        for (var j: i32 = -1; j <= 1; j++) {
            if (i == 0 && j == 0) {
                continue;
            }
            let neighbor = vec2<i32>(x.x + i, x.y + j);
            let neighbor_level = textureLoad(levelset, neighbor).r;
            if (sign(level) == sign(neighbor_level)) {
                continue;
            }
            let distance_to_level_zero = level / (level - neighbor_level);

            if (abs(distance_to_level_zero) < min_distance) {
                min_distance = abs(distance_to_level_zero);
                min_distance_seed = vec2<f32>(neighbor) + vec2<f32>(distance_to_level_zero * f32(i), distance_to_level_zero * f32(j));
            }
        }
    }

    textureStore(seeds, x, vec4<f32>(min_distance_seed, 0, 0));
}