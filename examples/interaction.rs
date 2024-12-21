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
    add_force::AddForceMaterial, advection::AdvectionMaterial, fluid_material::VelocityMaterial,
    uniform::SimulationUniform, FluidPlugin,
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
    .add_systems(Update, (on_advection_initialized, update))
    .add_systems(Update, mouse_motion);

    app.run();
}

fn setup_scene(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(Name::new("Camera"));

    commands.spawn(SimulationUniform {
        dx: 1.0f32,
        dt: 0.5f32,
        rho: 1.293f32,
    });
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
}

fn update(mut query: Query<&mut SimulationUniform>, _time: Res<Time>) {
    for mut uniform in &mut query {
        uniform.dt = 0.5;
    }
}

fn mouse_motion(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    touches: Res<Touches>,
    mut force_material: ResMut<AddForceMaterial>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<&OrthographicProjection, With<Camera2d>>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        let window = q_window.single();
        if let Some(cursor_position) = window.cursor_position() {
            let force = mouse_motion
                .read()
                .map(|mouse| mouse.delta)
                .collect::<Vec<_>>();

            let position = screen_to_mesh_coordinate(
                cursor_position,
                window,
                q_camera.single(),
                Vec2::splat(512.),
            );
            let position = vec![position; force.len()];
            force_material.force = force;
            force_material.position = position;

            return;
        }
    }

    let touch_forces = touches
        .iter()
        .map(|touch| touch.delta())
        .collect::<Vec<_>>();
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
    force_material.force = touch_forces;
    force_material.position = touch_position;
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
