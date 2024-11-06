#import bevy_fluid::coordinate::{left, right, bottom, top};

@group(0) @binding(0) var seeds: texture_storage_2d<rg32float, read_write>;
@group(1) @binding(0) var levelset: texture_storage_2d<r32float, read_write>;

@compute
@workgroup_size(8, 8, 1)
fn initialize(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let level = textureLoad(levelset, x).r;
    let level_left = textureLoad(levelset, left(x)).r;
    let level_right = textureLoad(levelset, right(x)).r;
    let level_bottom = textureLoad(levelset, bottom(x)).r;
    let level_top = textureLoad(levelset, top(x)).r;
    
    if (sign(level) == 0.0) {
        textureStore(seeds, x, vec4<f32>(-1, -1, 0, 0));
        return;
    } //else if (
    //     // sign(level) != sign(level_left) ||
    //     sign(level) != sign(level_right) // ||
    //     // sign(level) != sign(level_bottom) // ||
    //     // sign(level) != sign(level_top)
    // ) {
    //     distance = level / (level - level_right);
    //     textureStore(seeds, x, vec4<i32>(x, 0, 0));
    // } else {
    //     textureStore(seeds, x, vec4<i32>(-1, -1, 0, 0));
    // }

    // if (sign(level) == sign(level_right) && sign(level) == sign(level_top)) {
    //     textureStore(seeds, x, vec4<f32>(-1.0, -1.0, 0, 0));
    //     return;
    // }

    // var distance_to_level_zero_right = 1.0;
    // if (sign(level) != sign(level_right)) {
    //     distance_to_level_zero_right = level / (level - level_right);
    // }

    // var distance_to_level_zero_top = 1.0;
    // if (sign(level) != sign(level_top)) {
    //     distance_to_level_zero_top = level / (level - level_top);
    // }

    // if (abs(distance_to_level_zero_right) < abs(distance_to_level_zero_top)) {
    //     let seed = vec2<f32>(x) + vec2<f32>(distance_to_level_zero_right, 0.0);
    //     textureStore(seeds, x, vec4<f32>(seed, 0, 0));
    // } else {
    //     let seed = vec2<f32>(x) + vec2<f32>(0.0, distance_to_level_zero_top);
    //     textureStore(seeds, x, vec4<f32>(seed, 0, 0));
    // }

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