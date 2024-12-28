struct Circle {
    radius: f32,
    center: vec2<f32>,
    velocity: vec2<f32>,
}
@group(0) @binding(0) var u0: texture_storage_2d<r32float, read_write>;
@group(0) @binding(1) var v0: texture_storage_2d<r32float, read_write>;

@group(1) @binding(3) var grid_label: texture_storage_2d<r32uint, read_write>;

@group(2) @binding(0) var<storage, read> circles: array<Circle>;

@compute
@workgroup_size(8, 8, 1)
fn update_grid_label(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = vec2<i32>(i32(global_id.x), i32(global_id.y));
    let dim_grid = textureDimensions(grid_label);

    // ToDo: User defined boundary conditions
    if (x.x == 0 || x.x == i32(dim_grid.x) - 1 || x.y == 0 || x.y == i32(dim_grid.y) - 1) {
        textureStore(grid_label, x, vec4<u32>(2, 0, 0, 0));
        textureStore(u0, x, vec4<f32>(0, 0, 0, 0));
        textureStore(v0, x, vec4<f32>(0, 0, 0, 0));
        return;
    }
    
    let total = arrayLength(&circles);

    var i = 0u;
    var label = 1u;
    var u = 0.0;
    var v = 0.0;
    loop {
        if (i >= total) {
            break;
        }
        let circle = circles[i];
        let dx = f32(x.x) - circle.center.x;
        let dy = f32(x.y) - circle.center.y;
        let distance = sqrt(dx * dx + dy * dy);

        if distance < circle.radius {
            label = 2u;
            u = circle.velocity.x;
            v = circle.velocity.y;
        }

        continuing {
            i = i + 1u;
        }
    }
    textureStore(grid_label, x, vec4<u32>(label, 0, 0, 0));

    if (label == 2u) {
        textureStore(u0, x, vec4<f32>(u, 0.0, 0.0, 0.0));
        textureStore(v0, x, vec4<f32>(v, 0.0, 0.0, 0.0));
    }
}