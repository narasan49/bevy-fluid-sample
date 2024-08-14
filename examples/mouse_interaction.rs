extern crate bevy_fluid;

// use advection_plugin::AdvectionPlugin;
use bevy::{
    asset::AssetMetaCheck,
    input::mouse::MouseMotion,
    prelude::*,
    render::{
        settings::{Backends, WgpuSettings},
        RenderPlugin,
    },
    sprite::MaterialMesh2dBundle,
    window::PrimaryWindow,
};

use bevy_fluid::euler_fluid::{
    add_force::AddForceMaterial, advection::AdvectionMaterial, fluid_material::VelocityMaterial,
    FluidPlugin,
};

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
    .add_systems(Startup, setup_scene)
    .add_systems(Update, on_advection_initialized)
    .add_systems(Update, mouse_motion);

    app.run();
}

fn setup_scene(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(Name::new("Camera"));
}

fn on_advection_initialized(
    mut commands: Commands,
    advection: Option<Res<AdvectionMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VelocityMaterial>>,
) {
    if let Some(advection) = advection {
        if advection.is_changed() {
            info!("prepare velocity texture");
            // spwan plane to visualize advection
            let mesh = meshes.add(Rectangle::default());
            let material = materials.add(VelocityMaterial {
                offset: 0.5,
                scale: 0.1,
                u: Some(advection.u_in.clone()),
                v: Some(advection.v_in.clone()),
            });

            commands.spawn(MaterialMesh2dBundle {
                mesh: mesh.into(),
                transform: Transform::default().with_scale(Vec3::splat(512.)),
                material,
                ..default()
            });
        }
    }
}

fn mouse_motion(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut force_material: ResMut<AddForceMaterial>,
    q_window: Query<&Window, With<PrimaryWindow>>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        if let Some(cursor_position) = q_window.single().cursor_position() {
            let force = mouse_motion
                .read()
                .map(|mouse| mouse.delta)
                .collect::<Vec<_>>();
            let position = vec![cursor_position; force.len()];
            force_material.force = force;
            force_material.position = position;

            return;
        }
    }

    force_material.force = vec![];
    force_material.position = vec![];
}
