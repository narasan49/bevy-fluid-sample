use bevy::{
    prelude::*,
    render::{
        render_graph,
        render_resource::{CachedPipelineState, ComputePassDescriptor, PipelineCache},
    },
};

use super::{
    definition::FluidSettings,
    fluid_bind_group::{FluidBindGroups, FluidPipelines},
};

const WORKGROUP_SIZE: u32 = 8;

enum State {
    Loading,
    Init,
    Update,
}

pub struct EulerFluidNode {
    state: State,
    // Query BindGroups components
    // Reference: bevy\crates\bevy_ui\src\render\render_pass.rs
    query: QueryState<(Entity, &'static FluidSettings, &'static FluidBindGroups)>,
}

impl EulerFluidNode {
    pub fn new(world: &mut World) -> Self {
        Self {
            state: State::Loading,
            query: world.query_filtered(),
        }
    }
}

impl render_graph::Node for EulerFluidNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
        let pipelines = world.resource::<FluidPipelines>();
        let pipeline_cache = world.resource::<PipelineCache>();
        match self.state {
            State::Loading => {
                self.state = State::Init;
            }
            State::Init => {
                if let (
                    CachedPipelineState::Ok(_advection_pipeline),
                    CachedPipelineState::Ok(_add_force_pipeline),
                ) = (
                    pipeline_cache.get_compute_pipeline_state(pipelines.advection_pipeline),
                    pipeline_cache.get_compute_pipeline_state(pipelines.add_force_pipeline),
                ) {
                    self.state = State::Update;
                }
            }
            State::Update => {}
        }
    }
    fn run<'w>(
        &self,
        graph: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipelines = world.resource::<FluidPipelines>();
        match self.state {
            State::Loading => {}
            State::Init => {}
            State::Update => {
                let advection_pipeline = pipeline_cache
                    .get_compute_pipeline(pipelines.advection_pipeline)
                    .unwrap();
                let add_force_pipeline = pipeline_cache
                    .get_compute_pipeline(pipelines.add_force_pipeline)
                    .unwrap();
                for (entity, settings, bind_groups) in self.query.iter_manual(world) {
                    info!("running EulerFluidNode graph. {:?}", entity);
                    let mut pass = render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor::default());
                    let size = settings.size;

                    pass.set_pipeline(&advection_pipeline);
                    pass.set_bind_group(0, &bind_groups.velocity_bind_group, &[]);
                    pass.set_bind_group(
                        1,
                        &bind_groups.uniform_bind_group,
                        &[bind_groups.uniform_index],
                    );
                    pass.set_bind_group(2, &bind_groups.grid_center_bind_group, &[]);
                    pass.dispatch_workgroups(
                        size.0 + 1,
                        size.1 / WORKGROUP_SIZE / WORKGROUP_SIZE,
                        1,
                    );

                    pass.set_pipeline(&add_force_pipeline);
                    pass.set_bind_group(2, &bind_groups.local_forces_bind_group, &[]);
                    pass.dispatch_workgroups(
                        size.0 + 1,
                        size.1 / WORKGROUP_SIZE / WORKGROUP_SIZE,
                        1,
                    );
                }
            }
        }

        Ok(())
    }
}
