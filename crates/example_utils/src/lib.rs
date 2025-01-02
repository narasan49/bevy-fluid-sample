pub mod fps_counter;

use bevy::{input::mouse::MouseMotion, prelude::*, render::camera::CameraProjection, window::PrimaryWindow};
use bevy_fluid::euler_fluid::definition::{LocalForces, FluidSettings};

pub fn mouse_motion(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    touches: Res<Touches>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<&OrthographicProjection, With<Camera2d>>,
    mut q_fluid: Query<(&mut LocalForces, &FluidSettings, &Transform)>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        let window = q_window.single();
        if let Some(cursor_position) = window.cursor_position() {
            let force = mouse_motion
                .read()
                .map(|mouse| mouse.delta)
                .collect::<Vec<_>>();

            for (mut local_forces, settings, transform) in &mut q_fluid {
                let position = screen_to_mesh_coordinate(
                    cursor_position - transform.translation.xy() * Vec2::new(1.0, -1.0),
                    window,
                    q_camera.single(),
                    Vec2::new(settings.size.0 as f32, settings.size.1 as f32),
                );
                let position = vec![position; force.len()];
                local_forces.force = force.clone();
                local_forces.position = position;
            }
            return;
        }
    }

    let touch_forces = touches
        .iter()
        .map(|touch| touch.delta())
        .collect::<Vec<_>>();
    for (mut local_forces, settings, transform) in &mut q_fluid {
        let touch_position = touches
            .iter()
            .map(|touch| {
                screen_to_mesh_coordinate(
                    touch.position() - transform.translation.xy() * Vec2::new(1.0, -1.0),
                    q_window.single(),
                    q_camera.single(),
                    Vec2::new(settings.size.0 as f32, settings.size.1 as f32),
                )
            })
            .collect::<Vec<_>>();
        local_forces.force = touch_forces.clone();
        local_forces.position = touch_position;
    }
}

pub fn screen_to_mesh_coordinate(
    position: Vec2,
    window: &Window,
    projection: &OrthographicProjection,
    scale: Vec2,
) -> Vec2 {
    let window_size = window.size();
    let normalized_position = 2.0 * (position - window_size) / window_size + 1.0;
    let inv_proj = projection.get_clip_from_view().inverse();

    let position_on_mesh = inv_proj.mul_vec4(Vec4::new(
        normalized_position.x,
        normalized_position.y,
        0.0,
        1.0,
    ));

    position_on_mesh.xy() + 0.5 * scale
}