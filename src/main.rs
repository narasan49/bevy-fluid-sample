mod advection_plugin;
mod euler_fluid;
mod texture;
mod ui;

// use advection_plugin::AdvectionPlugin;
use bevy::{
    asset::AssetMetaCheck,
    core::FrameCount,
    math::vec3,
    prelude::*,
    render::{
        settings::{Backends, WgpuSettings},
        RenderPlugin,
    },
};
use euler_fluid::{
    advection::AdvectionMaterial,
    fluid_material::FluidMaterial,
    geometry::{self},
    uniform::SimulationUniform,
    FluidPlugin,
};
use iyes_perf_ui::{entries::PerfUiCompleteBundle, PerfUiPlugin};
use rand::Rng;
use ui::{AddButton, GameUiPlugin, ResetButton};

const WIDTH: f32 = 1280.0;
const HEIGHT: f32 = 720.0;

fn main() {
    let mut app = App::new();
    // [workaround] Asset meta files cannot be found on browser.
    // see also: https://github.com/bevyengine/bevy/issues/10157
    let meta_check = if cfg!(target_arch = "wasm32") {
        AssetMetaCheck::Never
    } else {
        AssetMetaCheck::Always
    };

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
            })
            .set(AssetPlugin {
                meta_check,
                ..default()
            }),
    )
    .add_plugins(FluidPlugin)
    .add_plugins(GameUiPlugin)
    .add_systems(Startup, setup_scene)
    .add_systems(Update, (on_advection_initialized, update_geometry))
    .add_systems(Update, (button_update, add_object));

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

// ToDo: Support for variable FPS
fn update_geometry(
    frame: Res<FrameCount>,
    mut object_query: Query<(&geometry::Circle, &mut Transform, &mut geometry::Velocity)>,
    uniform_query: Query<&SimulationUniform>,
) {
    let dt = uniform_query.single().dt * 0.1;
    let t = frame.0 as f32 * dt;
    let freq = 1.0;
    for (_circle, mut transform, mut velocity) in &mut object_query {
        let u = 100.0 * freq * f32::cos(t * freq);
        velocity.u = u;
        transform.translation.x += u * dt;
    }
}

fn button_update(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<ResetButton>)>,
    object_query: Query<Entity, With<geometry::Circle>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            for entity in object_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn add_object(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<AddButton>)>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            let mut rng = rand::thread_rng();
            commands.spawn((
                geometry::Circle {
                    radius: rng.gen_range(10..50) as f32,
                },
                Transform::from_translation(vec3(
                    rng.gen_range(0..512) as f32,
                    0.0,
                    rng.gen_range(0..512) as f32,
                )),
                geometry::Velocity { u: 0.0, v: 0.0 },
            ));
        }
    }
}
