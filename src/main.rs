mod euler_fluid;
use bevy::prelude::*;
use euler_fluid::FluidPlugin;

const WIDTH: f32 = 1280.0;
const HEIGHT: f32 =720.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (WIDTH, HEIGHT).into(),
                title: "bevy fluid".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FluidPlugin)
        .add_systems(Startup, setup_scene)
        .run();
    
}

#[derive(Component)]
struct CameraMarker;

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraMarker,
    ))
    .insert(Name::new("Camera"));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    })
    .insert(Name::new("Light"));
}