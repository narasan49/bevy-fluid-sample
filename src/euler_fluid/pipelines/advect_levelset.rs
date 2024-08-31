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

use crate::euler_fluid::{
    materials::{levelset::LevelSetBindGroupLayout, staggered_velocity::VelocityBindGroupLayout},
    uniform::SimulationUniform,
};

#[derive(Resource)]
pub struct AdvectLevelsetPipeline {
    pub pipeline: CachedComputePipelineId,
    pub init_pipeline: CachedComputePipelineId,
}

impl FromWorld for AdvectLevelsetPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let velocity_bind_group_layout = &world.resource::<VelocityBindGroupLayout>().0;
        let levelset_bind_group_layout = &world.resource::<LevelSetBindGroupLayout>().0;
        let uniform_bind_group_layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<SimulationUniform>(false),
            ),
        );

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/advect_levelset.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("advect_levelset_pipeline")),
            layout: vec![
                velocity_bind_group_layout.clone(),
                levelset_bind_group_layout.clone(),
                uniform_bind_group_layout.clone(),
            ],
            push_constant_ranges: Vec::new(),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("advect_levelset"),
        });

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/initialize_grid_centered.wgsl");
        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("init_levelset_pipeline")),
            layout: vec![levelset_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("initialize"),
        });
        Self {
            pipeline,
            init_pipeline,
        }
    }
}
