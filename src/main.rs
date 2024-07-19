mod advection_plugin;
mod euler_fluid;
mod texture;

// use advection_plugin::AdvectionPlugin;
use bevy::{
    math::vec2, prelude::*, render::{
        settings::{Backends, WgpuSettings},
        RenderPlugin,
    }
};
use euler_fluid::{advection::AdvectionMaterial, fluid_material::FluidMaterial, geometry::{CircleCollectionMaterial, CrircleUniform}, FluidPlugin};
use iyes_perf_ui::{entries::PerfUiCompleteBundle, PerfUiPlugin};

const WIDTH: f32 = 1280.0;
const HEIGHT: f32 = 720.0;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (WIDTH, HEIGHT).into(),
                    title: "bevy fluid".to_string(),
                    ..default()
                }),
                ..default()
            })
            .set(RenderPlugin {
                render_creation: bevy::render::settings::RenderCreation::Automatic(WgpuSettings {
                    backends: Some(Backends::DX12 | Backends::BROWSER_WEBGPU),
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins(FluidPlugin)
    // .add_plugins(AdvectionPlugin)
    .add_systems(Startup, setup_scene)
    .add_systems(Update, (on_advection_initialized, update_geometry));

    if cfg!(target_os = "windows") {
        app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
            .add_plugins(PerfUiPlugin);
    }

    app.run();
}

#[derive(Component)]
struct CameraMarker;

fn setup_scene(mut commands: Commands) {
    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 2.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            CameraMarker,
        ))
        .insert(Name::new("Camera"));

    commands
        .spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 1500.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        })
        .insert(Name::new("Light"));

    if cfg!(target_os = "windows") {
        commands.spawn(PerfUiCompleteBundle::default());
    }
}

fn on_advection_initialized(
    mut commands: Commands,
    advection: Option<Res<AdvectionMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FluidMaterial>>,
) {
    if let Some(advection) = advection {
        if advection.is_changed() {
            // spwan plane to visualize advection
            let mesh =
                meshes.add(Mesh::from(Plane3d::default()).translated_by(Vec3::new(-1.0, 0.0, 0.0)));
            let material = materials.add(FluidMaterial {
                base_color: LinearRgba::RED,
                velocity_texture: Some(advection.u_in.clone()),
            });
            commands.spawn((
                Name::new("advection"),
                MaterialMeshBundle {
                    mesh,
                    material,
                    ..default()
                },
            ));
        }
    }
}

fn update_geometry(
    mut geometry_collection: ResMut<CircleCollectionMaterial>,
    time: Res<Time>,
) {
    geometry_collection.circles = geometry_collection.circles.iter().map(|circle| {
        let x = circle.position.x;
        let new_x = 128.0 + 100.0 * f32::sin(time.elapsed_seconds());
        return CrircleUniform {
            position: vec2(new_x, circle.position.y),
            velocity: vec2((new_x - x) / time.delta_seconds(), 0.0),
            ..*circle
        }
    }).collect();
}