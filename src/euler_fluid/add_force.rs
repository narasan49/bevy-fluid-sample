use std::borrow::Cow;

use bevy::{
    prelude::*,
    render::{
        render_resource::{
            binding_types::uniform_buffer, BindGroupLayoutEntries, CachedComputePipelineId,
            ComputePipelineDescriptor, PipelineCache, ShaderStages,
        },
        renderer::RenderDevice,
    },
};

use super::{
    materials::{
        levelset::LevelSetBindGroupLayout, local_force::LocalForceBindGroupLayout,
        staggered_velocity::IntermediateVelocityBindGroupLayout,
    },
    uniform::SimulationUniform,
};

#[derive(Resource)]
pub struct AddForcePipeline {
    pub pipeline: CachedComputePipelineId,
}

impl FromWorld for AddForcePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = &world.resource::<LocalForceBindGroupLayout>().0;
        let intermediate_velocity_bind_group_layout =
            &world.resource::<IntermediateVelocityBindGroupLayout>().0;
        let uniform_bind_group_layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<SimulationUniform>(false),
            ),
        );
        let levelset_bindg_group_layout = &world.resource::<LevelSetBindGroupLayout>().0;

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/add_force.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![
                bind_group_layout.clone(),
                intermediate_velocity_bind_group_layout.clone(),
                uniform_bind_group_layout.clone(),
                levelset_bindg_group_layout.clone(),
            ],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("add_force"),
        });

        Self { pipeline }
    }
}
