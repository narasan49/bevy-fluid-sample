use bevy::{
    prelude::*,
    render::{
        render_graph,
        render_resource::{CachedPipelineState, PipelineCache},
    },
};

use super::fluid_bind_group::{FluidBindGroups, FluidPipelines};

enum State {
    Loading,
    Init,
    Update,
}

pub struct EulerFluidNode {
    state: State,
    // Query BindGroups components
    // Reference: bevy\crates\bevy_ui\src\render\render_pass.rs
    query: QueryState<(Entity, &'static FluidBindGroups)>,
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
                self.state = State::Loading;
            }
            State::Init => {
                if let (
                    CachedPipelineState::Ok(advection_pipeline),
                    CachedPipelineState::Ok(add_force_pipeline),
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
        for (entity, bind_groups) in self.query.iter_manual(world) {
            info!("running EulerFluidNode graph. {:?}", entity);
        }

        Ok(())
    }
}
