use bevy::{
    prelude::*,
    render::{
        render_graph::{self, RenderLabel},
        render_resource::{CachedPipelineState, ComputePassDescriptor, PipelineCache},
    },
};

use super::{
    definition::FluidSettings,
    fluid_bind_group::{
        FluidBindGroupResources, FluidBindGroups, FluidPipelines, JumpFloodingUniformBindGroups,
    },
};

const WORKGROUP_SIZE: u32 = 8;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(crate) struct FluidLabel;

enum State {
    Loading,
    Init,
    Update,
}

pub(crate) struct EulerFluidNode {
    state: State,
    // Query BindGroups components
    // Reference: bevy\crates\bevy_ui\src\render\render_pass.rs
    query: QueryState<(
        Entity,
        &'static FluidSettings,
        &'static FluidBindGroups,
        &'static JumpFloodingUniformBindGroups,
    )>,
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
                if let (
                    CachedPipelineState::Ok(_initialize_velocity_pipeline),
                    CachedPipelineState::Ok(_initialize_grid_center_pipeline),
                ) = (
                    pipeline_cache
                        .get_compute_pipeline_state(pipelines.initialize_velocity_pipeline),
                    pipeline_cache
                        .get_compute_pipeline_state(pipelines.initialize_grid_center_pipeline),
                ) {
                    self.state = State::Init;
                }
            }
            State::Init => {
                if let (
                    CachedPipelineState::Ok(_update_grid_label_pipeline),
                    CachedPipelineState::Ok(_advection_pipeline),
                    CachedPipelineState::Ok(_add_force_pipeline),
                    CachedPipelineState::Ok(_divergence_pipeline),
                    CachedPipelineState::Ok(_jacobi_iteration_pipeline),
                    CachedPipelineState::Ok(_jacobi_iteration_reverse_pipeline),
                    CachedPipelineState::Ok(_solve_velocity_pipeline),
                    CachedPipelineState::Ok(_recompute_levelset_initialization_pipeline),
                    CachedPipelineState::Ok(_recompute_levelset_iteration_pipeline),
                    CachedPipelineState::Ok(_recompute_levelset_solve_pipeline),
                    CachedPipelineState::Ok(_advect_levelset_pipeline),
                ) = (
                    pipeline_cache.get_compute_pipeline_state(pipelines.update_grid_label_pipeline),
                    pipeline_cache.get_compute_pipeline_state(pipelines.advection_pipeline),
                    pipeline_cache.get_compute_pipeline_state(pipelines.add_force_pipeline),
                    pipeline_cache.get_compute_pipeline_state(pipelines.divergence_pipeline),
                    pipeline_cache.get_compute_pipeline_state(pipelines.jacobi_iteration_pipeline),
                    pipeline_cache
                        .get_compute_pipeline_state(pipelines.jacobi_iteration_reverse_pipeline),
                    pipeline_cache.get_compute_pipeline_state(pipelines.solve_velocity_pipeline),
                    pipeline_cache.get_compute_pipeline_state(
                        pipelines.recompute_levelset_initialization_pipeline,
                    ),
                    pipeline_cache.get_compute_pipeline_state(
                        pipelines.recompute_levelset_iteration_pipeline,
                    ),
                    pipeline_cache
                        .get_compute_pipeline_state(pipelines.recompute_levelset_solve_pipeline),
                    pipeline_cache.get_compute_pipeline_state(pipelines.advect_levelset_pipeline),
                ) {
                    self.state = State::Update;
                }
            }
            State::Update => {}
        }
    }
    fn run<'w>(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipelines = world.resource::<FluidPipelines>();
        match self.state {
            State::Loading => {}
            State::Init => {
                let initialize_velocity_pipeline = pipeline_cache
                    .get_compute_pipeline(pipelines.initialize_velocity_pipeline)
                    .unwrap();
                let initialize_grid_center_pipeline = pipeline_cache
                    .get_compute_pipeline(pipelines.initialize_grid_center_pipeline)
                    .unwrap();
                for (_entity, settings, bind_groups, _) in self.query.iter_manual(world) {
                    let mut pass = render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor::default());
                    let size = settings.size;

                    pass.set_pipeline(&initialize_velocity_pipeline);
                    pass.set_bind_group(0, &bind_groups.velocity_bind_group, &[]);
                    pass.dispatch_workgroups(
                        size.0 + 1,
                        size.1 / WORKGROUP_SIZE / WORKGROUP_SIZE,
                        1,
                    );

                    pass.set_pipeline(&initialize_grid_center_pipeline);
                    pass.set_bind_group(0, &bind_groups.levelset_bind_group, &[]);
                    pass.set_bind_group(
                        1,
                        &bind_groups.uniform_bind_group,
                        &[bind_groups.uniform_index],
                    );
                    pass.dispatch_workgroups(size.0 / WORKGROUP_SIZE, size.1 / WORKGROUP_SIZE, 1);
                }
            }
            State::Update => {
                let update_grid_label_pipeline = pipeline_cache
                    .get_compute_pipeline(pipelines.update_grid_label_pipeline)
                    .unwrap();
                let advection_pipeline = pipeline_cache
                    .get_compute_pipeline(pipelines.advection_pipeline)
                    .unwrap();
                let add_force_pipeline = pipeline_cache
                    .get_compute_pipeline(pipelines.add_force_pipeline)
                    .unwrap();
                let divergence_pipeline = pipeline_cache
                    .get_compute_pipeline(pipelines.divergence_pipeline)
                    .unwrap();
                let jacobi_iteration_pipeline = pipeline_cache
                    .get_compute_pipeline(pipelines.jacobi_iteration_pipeline)
                    .unwrap();
                let jacobi_iteration_reverse_pipeline = pipeline_cache
                    .get_compute_pipeline(pipelines.jacobi_iteration_reverse_pipeline)
                    .unwrap();
                let solve_velocity_pipeline = pipeline_cache
                    .get_compute_pipeline(pipelines.solve_velocity_pipeline)
                    .unwrap();
                let recompute_levelset_initialization_pipeline = pipeline_cache
                    .get_compute_pipeline(pipelines.recompute_levelset_initialization_pipeline)
                    .unwrap();
                let recompute_levelset_itertation_pipeline = pipeline_cache
                    .get_compute_pipeline(pipelines.recompute_levelset_iteration_pipeline)
                    .unwrap();
                let recompute_levelset_solve_pipeline = pipeline_cache
                    .get_compute_pipeline(pipelines.recompute_levelset_solve_pipeline)
                    .unwrap();
                let advect_levelset_pipeline = pipeline_cache
                    .get_compute_pipeline(pipelines.advect_levelset_pipeline)
                    .unwrap();

                let bind_group_resources = world.resource::<FluidBindGroupResources>();
                for (_entity, settings, bind_groups, jump_flooding_uniform_bind_groups) in
                    self.query.iter_manual(world)
                {
                    let mut pass = render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor::default());
                    let size = settings.size;

                    pass.set_pipeline(&update_grid_label_pipeline);
                    pass.set_bind_group(0, &bind_groups.velocity_bind_group, &[]);
                    pass.set_bind_group(1, &bind_groups.levelset_bind_group, &[]);
                    pass.set_bind_group(2, &bind_group_resources.obstacles_bind_group, &[]);
                    pass.dispatch_workgroups(size.0 / WORKGROUP_SIZE, size.1 / WORKGROUP_SIZE, 1);

                    pass.set_pipeline(&advection_pipeline);
                    pass.set_bind_group(0, &bind_groups.velocity_bind_group, &[]);
                    pass.set_bind_group(1, &bind_groups.levelset_bind_group, &[]);
                    pass.set_bind_group(
                        2,
                        &bind_groups.uniform_bind_group,
                        &[bind_groups.uniform_index],
                    );
                    pass.dispatch_workgroups(
                        size.0 + 1,
                        size.1 / WORKGROUP_SIZE / WORKGROUP_SIZE,
                        1,
                    );

                    pass.set_pipeline(&add_force_pipeline);
                    pass.set_bind_group(
                        1,
                        &bind_groups.uniform_bind_group,
                        &[bind_groups.uniform_index],
                    );
                    pass.set_bind_group(2, &bind_groups.local_forces_bind_group, &[]);
                    pass.set_bind_group(3, &bind_groups.levelset_bind_group, &[]);
                    pass.dispatch_workgroups(
                        size.0 + 1,
                        size.1 / WORKGROUP_SIZE / WORKGROUP_SIZE,
                        1,
                    );

                    pass.set_pipeline(&divergence_pipeline);
                    pass.set_bind_group(1, &bind_groups.divergence_bind_group, &[]);
                    pass.set_bind_group(2, &bind_groups.levelset_bind_group, &[]);
                    pass.dispatch_workgroups(size.0 / WORKGROUP_SIZE, size.1 / WORKGROUP_SIZE, 1);

                    pass.set_bind_group(
                        0,
                        &bind_groups.uniform_bind_group,
                        &[bind_groups.uniform_index],
                    );
                    pass.set_bind_group(1, &bind_groups.pressure_bind_group, &[]);
                    pass.set_bind_group(2, &bind_groups.divergence_bind_group, &[]);
                    pass.set_bind_group(3, &bind_groups.levelset_bind_group, &[]);
                    for _ in 0..5 {
                        pass.set_pipeline(&jacobi_iteration_pipeline);
                        pass.dispatch_workgroups(
                            size.0 / WORKGROUP_SIZE,
                            size.1 / WORKGROUP_SIZE,
                            1,
                        );
                        pass.set_pipeline(&jacobi_iteration_reverse_pipeline);
                        pass.dispatch_workgroups(
                            size.0 / WORKGROUP_SIZE,
                            size.1 / WORKGROUP_SIZE,
                            1,
                        );
                    }

                    pass.set_pipeline(&solve_velocity_pipeline);
                    pass.set_bind_group(0, &bind_groups.velocity_bind_group, &[]);
                    pass.set_bind_group(
                        1,
                        &bind_groups.uniform_bind_group,
                        &[bind_groups.uniform_index],
                    );
                    pass.set_bind_group(2, &bind_groups.pressure_bind_group, &[]);
                    pass.set_bind_group(3, &bind_groups.levelset_bind_group, &[]);
                    pass.dispatch_workgroups(
                        size.0 + 1,
                        size.1 / WORKGROUP_SIZE / WORKGROUP_SIZE,
                        1,
                    );

                    // recompute levelset
                    pass.set_pipeline(&recompute_levelset_initialization_pipeline);
                    pass.set_bind_group(0, &bind_groups.levelset_bind_group, &[]);
                    pass.set_bind_group(1, &bind_groups.jump_flooding_seeds_bind_group, &[]);
                    pass.dispatch_workgroups(size.0 / WORKGROUP_SIZE, size.1 / WORKGROUP_SIZE, 1);

                    pass.set_pipeline(&recompute_levelset_itertation_pipeline);
                    pass.set_bind_group(0, &bind_groups.jump_flooding_seeds_bind_group, &[]);
                    for bind_group in
                        &jump_flooding_uniform_bind_groups.jump_flooding_step_bind_groups
                    {
                        pass.set_bind_group(1, bind_group, &[]);
                        pass.dispatch_workgroups(
                            size.0 / WORKGROUP_SIZE,
                            size.1 / WORKGROUP_SIZE,
                            1,
                        );
                    }

                    pass.set_pipeline(&recompute_levelset_solve_pipeline);
                    pass.set_bind_group(0, &bind_groups.levelset_bind_group, &[]);
                    pass.set_bind_group(1, &bind_groups.jump_flooding_seeds_bind_group, &[]);
                    pass.dispatch_workgroups(size.0 / WORKGROUP_SIZE, size.1 / WORKGROUP_SIZE, 1);

                    pass.set_pipeline(&advect_levelset_pipeline);
                    pass.set_bind_group(0, &bind_groups.velocity_bind_group, &[]);
                    pass.set_bind_group(1, &bind_groups.levelset_bind_group, &[]);
                    pass.set_bind_group(
                        2,
                        &bind_groups.uniform_bind_group,
                        &[bind_groups.uniform_index],
                    );
                    pass.dispatch_workgroups(size.0 / WORKGROUP_SIZE, size.1 / WORKGROUP_SIZE, 1);
                }
            }
        }

        Ok(())
    }
}
