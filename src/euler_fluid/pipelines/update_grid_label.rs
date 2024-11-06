use std::borrow::Cow;

use bevy::{
    prelude::*,
    render::render_resource::{CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache},
};

use crate::euler_fluid::{
    grid_label::GridLabelBindGroupLayout, materials::levelset::LevelSetBindGroupLayout,
};

pub struct UpdateGridLabelPipeline {
    pub pipeline: CachedComputePipelineId,
}

impl FromWorld for UpdateGridLabelPipeline {
    fn from_world(world: &mut World) -> Self {
        let grid_label_bind_group_layout = &world.resource::<GridLabelBindGroupLayout>().0;
        let levelset_bind_group_layout = &world.resource::<LevelSetBindGroupLayout>().0;

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/update_grid_label.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("update_grid_label_pipeline")),
            layout: vec![
                grid_label_bind_group_layout.clone(),
                levelset_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update_grid_label"),
        });

        Self { pipeline }
    }
}
