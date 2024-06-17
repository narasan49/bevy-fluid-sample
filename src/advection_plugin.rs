use bevy::{prelude::*, render::{extract_resource::ExtractResourcePlugin, graph::CameraDriverLabel, render_graph::{self, RenderGraph, RenderLabel}, render_resource::{CachedPipelineState, ComputePassDescriptor, PipelineCache, TextureFormat}, Render, RenderApp, RenderSet}};
use crate::{euler_fluid::{advection::{AdvectionMaterial, AdvectionPipeline, AdvectionBindGroup}, uniform::{SimulationUniform, SimulationUniformBindGroup}}, texture::ImageForCS};
use crate::euler_fluid;

const SIZE: (u32, u32) = (512, 512);
const SIZE_U: (u32, u32) = (SIZE.0 + 1, SIZE.1);
const SIZE_V: (u32, u32) = (SIZE.0, SIZE.1 + 1);
const WORKGROUP_SIZE: u32 = 64;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct AdvectionLabel;

pub struct AdvectionPlugin;

impl Plugin for AdvectionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(ExtractResourcePlugin::<AdvectionMaterial>::default())
            .add_systems(Startup, setup)
            .add_systems(Update, update);

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(Render, euler_fluid::advection::prepare_bind_group.in_set(RenderSet::PrepareBindGroups));

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node(AdvectionLabel, AdvectionNode::default());
        render_graph.add_node_edge(AdvectionLabel, CameraDriverLabel);
    }

    // finish build
    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<AdvectionPipeline>();
    }
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let u_in = Image::new_texture_storage(SIZE_U, TextureFormat::R32Float);
    let u_in = images.add(u_in);

    let u_out = Image::new_texture_storage(SIZE_U, TextureFormat::R32Float);
    let u_out = images.add(u_out);
    
    let v_in = Image::new_texture_storage(SIZE_V, TextureFormat::R32Float);
    let v_in = images.add(v_in);

    let v_out = Image::new_texture_storage(SIZE_V, TextureFormat::R32Float);
    let v_out = images.add(v_out);

    commands.insert_resource(AdvectionMaterial { u_in, u_out, v_in, v_out});
    commands.spawn(SimulationUniform{ dt: 0.1f32, dx: 1.0f32, rho: 1.0f32 });
}

fn update(
    mut query: Query<&mut SimulationUniform>,
) {
    for mut uniform in &mut query {
        uniform.dt = 1.0;
    }
}

enum AdvectionState {
    Loading,
    Init,
    Update,
}

struct AdvectionNode {
    state: AdvectionState,
}

impl Default for AdvectionNode {
    fn default() -> Self {
        Self {
            state: AdvectionState::Loading,
        }
    }
}

impl render_graph::Node for AdvectionNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<AdvectionPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        match self.state {
            AdvectionState::Loading => {
                if let CachedPipelineState::Ok(_) = pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline) {
                    self.state = AdvectionState::Init;
                }
            }
            AdvectionState::Init => {
                if let CachedPipelineState::Ok(_) = pipeline_cache.get_compute_pipeline_state(pipeline.pipeline) {
                    self.state = AdvectionState::Update;
                }
            }
            AdvectionState::Update => {}
        }
    }

    fn run<'w>(
            &self,
            _graph: &mut render_graph::RenderGraphContext,
            render_context: &mut bevy::render::renderer::RenderContext<'w>,
            world: &'w World,
        ) -> Result<(), render_graph::NodeRunError> {
        let bind_group = &world.resource::<AdvectionBindGroup>().0;
        let uniform_bind_group = &world.resource::<SimulationUniformBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<AdvectionPipeline>();

        let mut pass = render_context.command_encoder().begin_compute_pass(&ComputePassDescriptor::default());
        pass.set_bind_group(0, bind_group, &[]);
        pass.set_bind_group(1, uniform_bind_group, &[]);

        match self.state {
            AdvectionState::Loading => {}
            AdvectionState::Init => {
                let init_pipeline = pipeline_cache.get_compute_pipeline(pipeline.init_pipeline).unwrap();
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 + 1, 1);
            }
            AdvectionState::Update => {
                let update_pipeline = pipeline_cache.get_compute_pipeline(pipeline.pipeline).unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 + 1, 1);

                let swap_pipeline = pipeline_cache.get_compute_pipeline(pipeline.swap_pipeline).unwrap();
                pass.set_pipeline(swap_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 + 1, 1);
                
            }
        }
        Ok(())
    }
}