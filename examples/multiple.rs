extern crate bevy_fluid;

use bevy::{
    asset::AssetMetaCheck,
    prelude::*,
    render::{
        settings::{Backends, RenderCreation, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_fluid::euler_fluid::{
    definition::{FluidSettings, SimulationInterval},
    uniform::SimulationUniform,
    FluidPlugin,
};

const WIDTH: f32 = 1280.0;
const HEIGHT: f32 = 720.0;

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
        .add_systems(Startup, setup_scene)
        .run();
}

fn setup_scene(mut commands: Commands) {
    info!("initialize scene.");
    commands.spawn(FluidSettings {
        dx: 1.0f32,
        dt: SimulationInterval::Fixed(0.5f32),
        rho: 1.293f32,
        size: (512, 512),
    });

    commands.spawn(SimulationUniform {
        dx: 1.0f32,
        dt: 0.5f32,
        rho: 1.293f32,
    });
}
