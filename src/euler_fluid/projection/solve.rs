use std::borrow::Cow;

use bevy::{
    asset::AssetServer,
    prelude::*,
    render::{
        render_resource::{
            binding_types::uniform_buffer, AsBindGroup, BindGroupLayoutEntries,
            CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache, ShaderStages,
        },
        renderer::RenderDevice,
    },
};

use crate::euler_fluid::{
    grid_label::GridLabelMaterial,
    materials::{pressure::PressureMaterial, staggered_velocity::{IntermediateVelocityBindGroupLayout, VelocityBindGroupLayout}},
    uniform::SimulationUniform,
};

#[derive(Resource)]
pub struct SolvePressurePipeline {
    pub pipeline: CachedComputePipelineId,
}

impl FromWorld for SolvePressurePipeline {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let velocity_bind_group_layout = &world.resource::<VelocityBindGroupLayout>().0;
        let intermediate_velocity_bind_group_layout =
            &world.resource::<IntermediateVelocityBindGroupLayout>().0;
        let pressure_bind_group_layout = PressureMaterial::bind_group_layout(render_device);

        let grid_label_bind_group_layout = GridLabelMaterial::bind_group_layout(render_device);
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/solve_pressure.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();

        let uniform_bind_group_layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<SimulationUniform>(false),
            ),
        );

        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("Solve Pressure")),
            layout: vec![
                velocity_bind_group_layout.clone(),
                intermediate_velocity_bind_group_layout.clone(),
                pressure_bind_group_layout.clone(),
                uniform_bind_group_layout.clone(),
                grid_label_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("solve_pressure"),
        });

        Self { pipeline }
    }
}
