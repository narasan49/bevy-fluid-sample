extern crate bevy_fluid;

use bevy::{
    asset::AssetMetaCheck,
    input::mouse::MouseMotion,
    prelude::*,
    render::{
        camera::CameraProjection,
        settings::{Backends, WgpuSettings},
        RenderPlugin,
    },
    sprite::MaterialMesh2dBundle,
    window::PrimaryWindow,
};

use bevy_fluid::euler_fluid::{
    definition::{FluidSettings, LocalForces, SimulationInterval, VelocityTextures},
    fluid_material::VelocityMaterial,
    FluidPlugin,
};

const WIDTH: f32 = 1280.0;
const HEIGHT: f32 = 720.0;

#[derive(Component)]
struct MeshMarker;

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
                    fit_canvas_to_parent: true,
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

    commands.spawn(FluidSettings {
        dx: 1.0f32,
        dt: SimulationInterval::Fixed(0.5f32),
        rho: 1.293f32, // water
        size: (512, 512),
    });
}

fn on_advection_initialized(
    mut commands: Commands,
    query: Query<&VelocityTextures, Added<VelocityTextures>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VelocityMaterial>>,
) {
    for velocity_texture in &query {
        info!("prepare velocity texture");
        // spwan plane to visualize advection
        let mesh = meshes.add(Rectangle::default());
        let material = materials.add(VelocityMaterial {
            offset: 0.5,
            scale: 0.1,
            u: Some(velocity_texture.u0.clone()),
            v: Some(velocity_texture.v0.clone()),
        });

        commands
            .spawn(MaterialMesh2dBundle {
                mesh: mesh.into(),
                transform: Transform::default().with_scale(Vec3::splat(512.0)),
                material,
                ..default()
            })
            .insert(MeshMarker);
    }
}

fn mouse_motion(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    touches: Res<Touches>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<&OrthographicProjection, With<Camera2d>>,
    mut q_fluid: Query<&mut LocalForces>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        let window = q_window.single();
        if let Some(cursor_position) = window.cursor_position() {
            let force = mouse_motion
                .read()
                .map(|mouse| mouse.delta)
                .collect::<Vec<_>>();

            for mut local_forces in &mut q_fluid {
                let position = screen_to_mesh_coordinate(
                    cursor_position,
                    window,
                    q_camera.single(),
                    Vec2::splat(512.),
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
    for mut local_forces in &mut q_fluid {
        let touch_position = touches
            .iter()
            .map(|touch| {
                screen_to_mesh_coordinate(
                    touch.position(),
                    q_window.single(),
                    q_camera.single(),
                    Vec2::splat(512.),
                )
            })
            .collect::<Vec<_>>();
        local_forces.force = touch_forces.clone();
        local_forces.position = touch_position;
    }
}

fn screen_to_mesh_coordinate(
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
