pub mod definition;
pub mod fluid_bind_group;
pub mod fluid_material;
pub mod geometry;
pub mod render_node;
pub mod setup_components;

use crate::euler_fluid::definition::{FluidSettings, LevelsetTextures};
use crate::euler_fluid::fluid_bind_group::FluidBindGroups;
use bevy::{
    asset::load_internal_asset,
    math::vec2,
    prelude::*,
    render::{
        extract_component::{ExtractComponentPlugin, UniformComponentPlugin},
        extract_resource::ExtractResourcePlugin,
        graph::CameraDriverLabel,
        render_graph::RenderGraph,
        Render, RenderApp, RenderSet,
    },
    sprite::Material2dPlugin,
};
use definition::{
    CircleObstacle, DivergenceTextures, JumpFloodingSeedsTextures, LocalForces, Obstacles,
    PressureTextures, SimulationUniform, VelocityTextures,
};
use fluid_bind_group::FluidPipelines;
use fluid_material::VelocityMaterial;
use geometry::Velocity;

use render_node::{EulerFluidNode, FluidLabel};

use setup_components::watch_fluid_component;

const FLUID_UNIFORM_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x8B9323522322463BA8CF530771C532EF);

const COORDINATE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x9F8E2E5B1E5F40C096C31175C285BF11);

pub struct FluidPlugin;

impl Plugin for FluidPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<Obstacles>::default())
            .add_plugins(ExtractComponentPlugin::<FluidSettings>::default())
            .add_plugins(ExtractComponentPlugin::<FluidBindGroups>::default())
            .add_plugins(ExtractComponentPlugin::<VelocityTextures>::default())
            .add_plugins(ExtractComponentPlugin::<PressureTextures>::default())
            .add_plugins(ExtractComponentPlugin::<DivergenceTextures>::default())
            .add_plugins(ExtractComponentPlugin::<LevelsetTextures>::default())
            .add_plugins(ExtractComponentPlugin::<JumpFloodingSeedsTextures>::default())
            .add_plugins(ExtractComponentPlugin::<LocalForces>::default())
            .add_plugins(ExtractComponentPlugin::<SimulationUniform>::default())
            .add_plugins(UniformComponentPlugin::<SimulationUniform>::default())
            .add_plugins(MaterialPlugin::<VelocityMaterial>::default())
            .add_plugins(Material2dPlugin::<VelocityMaterial>::default())
            .add_systems(Update, update_geometry)
            .add_systems(Update, watch_fluid_component);

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(
                Render,
                fluid_bind_group::prepare_resource_recompute_levelset
                    .in_set(RenderSet::PrepareResources),
            )
            .add_systems(
                Render,
                fluid_bind_group::prepare_fluid_bind_groups.in_set(RenderSet::PrepareBindGroups),
            )
            .add_systems(
                Render,
                fluid_bind_group::prepare_fluid_bind_group_for_resources
                    .in_set(RenderSet::PrepareBindGroups),
            );

        let mut world = render_app.world_mut();
        let euler_fluid_node = EulerFluidNode::new(&mut world);
        let mut render_graph = world.resource_mut::<RenderGraph>();
        render_graph.add_node(FluidLabel, euler_fluid_node);
        render_graph.add_node_edge(FluidLabel, CameraDriverLabel);

        load_internal_asset!(
            app,
            FLUID_UNIFORM_SHADER_HANDLE,
            "../assets/shaders/fluid_uniform.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            COORDINATE_SHADER_HANDLE,
            "../assets/shaders/coordinate.wgsl",
            Shader::from_wgsl
        )
    }

    fn finish(&self, app: &mut App) {
        app.init_resource::<Obstacles>();

        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<FluidPipelines>();
    }
}

fn update_geometry(
    query: Query<(&geometry::Circle, &Transform, &Velocity)>,
    mut obstacles: ResMut<Obstacles>,
) {
    let circles = query
        .iter()
        .map(|(circle, transform, velocity)| {
            return CircleObstacle {
                radius: circle.radius,
                center: transform.translation.xz(),
                velocity: vec2(velocity.u, velocity.v),
            };
        })
        .collect::<Vec<_>>();

    obstacles.circles = circles;
}
