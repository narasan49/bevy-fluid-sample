struct Circle {
    radius: f32,
    center: vec2<f32>,
    velocity: vec2<f32>,
}
@group(0) @binding(0) var grid_label: texture_storage_2d<r32uint, read_write>;
@group(0) @binding(1) var u_solid: texture_storage_2d<r32float, read_write>;
@group(0) @binding(2) var v_solid: texture_storage_2d<r32float, read_write>;

@group(1) @binding(0) var<storage, read> circles: array<Circle>;


@compute
@workgroup_size(8, 8, 1)
fn initialize(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = vec2<i32>(i32(global_id.x), i32(global_id.y));
    textureStore(grid_label, x, vec4<u32>(1, 0, 0, 0));
    textureStore(u_solid, x, vec4<f32>(0, 0, 0, 0));
    textureStore(v_solid, x, vec4<f32>(0, 0, 0, 0));
}

@compute
@workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = vec2<i32>(i32(global_id.x), i32(global_id.y));
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
    textureStore(u_solid, x, vec4<f32>(u, 0.0, 0.0, 0.0));
    textureStore(v_solid, x, vec4<f32>(v, 0.0, 0.0, 0.0));
}