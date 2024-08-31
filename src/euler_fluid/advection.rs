use std::borrow::Cow;

use bevy::{
    prelude::*,
    render::{
        render_resource::{
            binding_types::uniform_buffer, AsBindGroup, BindGroupLayoutEntries,
            CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache, ShaderStages,
        },
        renderer::RenderDevice,
    },
};

use super::{
    grid_label::GridLabelMaterial,
    materials::staggered_velocity::{IntermediateVelocityBindGroupLayout, VelocityBindGroupLayout},
    uniform::SimulationUniform,
};

#[derive(Resource)]
pub struct AdvectionPipeline {
    pub init_pipeline: CachedComputePipelineId,
    pub pipeline: CachedComputePipelineId,
}

impl FromWorld for AdvectionPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let velocity_bind_group_layout = &world.resource::<VelocityBindGroupLayout>().0;
        let intermediate_velocity_bind_group_layout =
            &world.resource::<IntermediateVelocityBindGroupLayout>().0;
        let uniform_bind_group_layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<SimulationUniform>(false),
            ),
        );
        let grid_label_bind_group_layout = GridLabelMaterial::bind_group_layout(render_device);

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/advection.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![
                velocity_bind_group_layout.clone(),
                intermediate_velocity_bind_group_layout.clone(),
                uniform_bind_group_layout.clone(),
                grid_label_bind_group_layout.clone(),
            ],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("advection"),
        });

        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![
                velocity_bind_group_layout.clone(),
                intermediate_velocity_bind_group_layout.clone(),
                uniform_bind_group_layout.clone(),
                grid_label_bind_group_layout.clone(),
            ],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("initialize"),
        });

        Self {
            init_pipeline,
            pipeline,
        }
    }
}
