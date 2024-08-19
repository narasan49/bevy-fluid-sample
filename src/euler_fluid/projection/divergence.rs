use std::borrow::Cow;

use bevy::{
    asset::AssetServer,
    prelude::*,
    render::{
        render_resource::{
            AsBindGroup, CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache,
        },
        renderer::RenderDevice,
    },
};

use crate::euler_fluid::{
    grid_label::GridLabelMaterial,
    materials::{
        divergence::DivergenceBindGroupLayout,
        staggered_velocity::IntermediateVelocityBindGroupLayout,
    },
};

#[derive(Resource)]
pub struct DivergencePipeline {
    pub pipeline: CachedComputePipelineId,
}

impl FromWorld for DivergencePipeline {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let intermediate_velocity_bind_group_layout =
            &world.resource::<IntermediateVelocityBindGroupLayout>().0;
        let bind_group_layout = &world.resource::<DivergenceBindGroupLayout>().0;
        let grid_label_bind_group_layout = GridLabelMaterial::bind_group_layout(render_device);
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/divergence.wgsl");

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![
                bind_group_layout.clone(),
                intermediate_velocity_bind_group_layout.clone(),
                grid_label_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("divergence"),
        });

        Self { pipeline }
    }
}
