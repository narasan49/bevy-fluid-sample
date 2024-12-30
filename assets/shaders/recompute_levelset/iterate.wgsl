@group(0) @binding(1) var seeds: texture_storage_2d<rg32float, read_write>;

@group(1) @binding(0) var<uniform> step: i32;

@compute
@workgroup_size(8, 8, 1)
fn iterate(
    @builtin(global_invocation_id) invocation_id: vec3<u32>
) {
    let x = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let current_seed = textureLoad(seeds, x);
    for (var i: i32 = -1; i <= 1; i++) {
        for (var j: i32 = -1; j <= 1; j++) {
            if (i == 0 && j == 0) {
                continue;
            }
            let neighbor = vec2<i32>(x.x + i * step, x.y + j * step);
            let neighbor_seed = textureLoad(seeds, neighbor);
            if (neighbor_seed.x == -1.0 && neighbor_seed.y == -1.0) {
                continue;
            }
            
            if (current_seed.x == -1.0 && current_seed.y == -1.0) {
                textureStore(seeds, x, neighbor_seed);
            } else {
                let distance_to_seed = distance(vec2<f32>(current_seed.xy), vec2<f32>(x));
                let distance_to_neighbor = distance(vec2<f32>(neighbor_seed.xy), vec2<f32>(x));
                if (distance_to_neighbor < distance_to_seed) {
                    textureStore(seeds, x, neighbor_seed);
                }
            }
        }
    }
}