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

    if (level == 0.0) {
        textureStore(seeds, x, vec4<f32>(vec2<f32>(x), 0, 0));
        return;
    }

    // find the point to intersect the zero level set
    let dim = vec2<i32>(textureDimensions(levelset));
    // array can be looped over only with a constant index
    // let neibors = array<vec2<i32>, 4>(
    //     x + vec2<i32>(-1, 0),
    //     x + vec2<i32>(1, 0),
    //     x + vec2<i32>(0, -1),
    //     x + vec2<i32>(0, 1)
    // );

    // ToDo: Condider if the result is better when using 8 neighbors
    for (var k: i32 = 0; k < 4; k++) {
        let i = select(-1, 1, k / 2 == 0);
        let j = select(-1, 1, k / 2 == 1);
        let neighbor = x + vec2<i32>(i, j);
        if (neighbor.x < 0 || neighbor.y < 0 || neighbor.x >= dim.x || neighbor.y >= dim.y) {
            continue;
        }
        let neighbor_level = textureLoad(levelset, neighbor).r;
        if (sign(level) == sign(neighbor_level)) {
            continue;
        }
        let distance_to_level_zero = level / (level - neighbor_level);

        if (abs(distance_to_level_zero) < min_distance) {
            min_distance = abs(distance_to_level_zero);
            min_distance_seed = vec2<f32>(x) + vec2<f32>(distance_to_level_zero * f32(i), distance_to_level_zero * f32(j));
        }
    }
    
    textureStore(seeds, x, vec4<f32>(min_distance_seed, 0, 0));
}