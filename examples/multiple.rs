extern crate bevy_eulerian_fluid;

use bevy::{
    asset::AssetMetaCheck,
    prelude::*,
    render::{
        settings::{Backends, RenderCreation, WgpuSettings},
        RenderPlugin,
    },
    sprite::MaterialMesh2dBundle,
};
use bevy_eulerian_fluid::{
    definition::{FluidSettings, VelocityTextures},
    fluid_material::VelocityMaterial,
    FluidPlugin,
};
use example_utils::{fps_counter::FpsCounterPlugin, mouse_motion};

const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 360.0;

fn main() {
    // [workaround] Asset meta files cannot be found on browser.
    // see also: https://github.com/bevyengine/bevy/issues/10157
    let meta_check = if cfg!(target_arch = "wasm32") {
        AssetMetaCheck::Never
    } else {
        AssetMetaCheck::Always
    };

    let _app = App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (WIDTH, HEIGHT).into(),
                        title: "fluid component".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
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
        .add_plugins(FpsCounterPlugin)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (mouse_motion, on_fluid_setup))
        .run();
}

fn setup_scene(mut commands: Commands) {
    info!("initialize scene.");
    commands
        .spawn(Camera2dBundle::default())
        .insert(Name::new("Camera"));

    let size = 128u32;
    for i in 0..4 {
        for j in 0..2 {
            let translation = Vec3::new(
                (i * size) as f32 * 1.1 - size as f32 * 1.6,
                (j * size) as f32 * 1.1 - size as f32 * 0.8,
                0.0,
            );
            commands
                .spawn(FluidSettings {
                    dx: 1.0f32,
                    dt: 0.5f32,
                    rho: 1.293f32,
                    gravity: Vec2::ZERO,
                    size: (size, size),
                    initial_fluid_level: 1.0f32,
                })
                .insert(
                    Transform::default()
                        .with_scale(Vec3::splat(size as f32))
                        .with_translation(translation),
                );
        }
    }
}

fn on_fluid_setup(
    mut commands: Commands,
    query: Query<(&VelocityTextures, &Transform), Added<VelocityTextures>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VelocityMaterial>>,
) {
    for (velocity_texture, transform) in &query {
        // spwan plane to visualize advection
        let mesh = meshes.add(Rectangle::default());
        let material = materials.add(VelocityMaterial {
            offset: 0.5,
            scale: 0.1,
            u: Some(velocity_texture.u0.clone()),
            v: Some(velocity_texture.v0.clone()),
        });

        commands.spawn(MaterialMesh2dBundle {
            mesh: mesh.into(),
            transform: *transform,
            material,
            ..default()
        });
    }
}
