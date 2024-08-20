use std::borrow::Cow;

use bevy::{
    asset::AssetServer,
    prelude::*,
    render::render_resource::{
        binding_types::uniform_buffer, AsBindGroup, BindGroupLayoutEntries,
        CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache, ShaderStages,
    },
};

use crate::euler_fluid::{
    grid_label::GridLabelMaterial,
    materials::{
        divergence::DivergenceBindGroupLayout,
        pressure::{IntermediatePressureBindGroupLayout, PressureBindGroupLayout},
    },
    uniform::SimulationUniform,
};

#[derive(Resource)]
pub struct JacobiPipeline {
    pub pipeline: CachedComputePipelineId,
    pub swap_pipeline: CachedComputePipelineId,
}

impl FromWorld for JacobiPipeline {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let render_device = world.resource();
        let divergence_bind_group_layout = &world.resource_ref::<DivergenceBindGroupLayout>().0;
        let pressure_bind_group_layout = &world.resource::<PressureBindGroupLayout>().0;
        let intermediate_pressure_bind_group_layout =
            &world.resource::<IntermediatePressureBindGroupLayout>().0;
        let grid_label_bind_group_layout = GridLabelMaterial::bind_group_layout(render_device);

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/jacobi_iteration.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();

        let uniform_bind_group_layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<SimulationUniform>(false),
            ),
        );

        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![
                pressure_bind_group_layout.clone(),
                intermediate_pressure_bind_group_layout.clone(),
                uniform_bind_group_layout.clone(),
                grid_label_bind_group_layout.clone(),
                divergence_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("jacobi_iteration"),
        });

        let swap_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![
                pressure_bind_group_layout.clone(),
                intermediate_pressure_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("swap_buffers"),
        });

        Self {
            pipeline,
            swap_pipeline,
        }
    }
}
