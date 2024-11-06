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

use crate::euler_fluid::materials::{
    jfa_seeds::JumpFloodingSeedBindGroupLayout, jump_flooding_uniform::JumpFloodingUniform,
    levelset::LevelSetBindGroupLayout,
};

#[derive(Resource)]
pub struct RecomputeLevelsetPipeline {
    pub initialize_pipeline: CachedComputePipelineId,
    pub jump_flood_pipeline: CachedComputePipelineId,
    pub sdf_pipeline: CachedComputePipelineId,
}

impl FromWorld for RecomputeLevelsetPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let jump_flooding_uniform = render_device.create_bind_group_layout(
            Some("JumpFloodingUniformBindGroupLayout"),
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<JumpFloodingUniform>(false),
            ),
        );
        let jump_flooding_seed_bind_group_layout =
            &world.resource::<JumpFloodingSeedBindGroupLayout>().0;
        // let jump_flooding_uniform = &world.resource::<JumpFloodingUniformBindGroupLayout>().0;
        let levelset_bind_group_layout = &world.resource::<LevelSetBindGroupLayout>().0;

        let pipeline_cache = world.resource::<PipelineCache>();

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/recompute_levelset/initialize_seeds.wgsl");

        let initialize_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("RecomputeLevelset initialize_pipeline")),
                layout: vec![
                    jump_flooding_seed_bind_group_layout.clone(),
                    levelset_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader,
                shader_defs: vec![],
                entry_point: Cow::from("initialize"),
            });

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/recompute_levelset/jump_flooding.wgsl");

        let jump_flood_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("RecomputeLevelset jump_flood_pipeline")),
                layout: vec![
                    jump_flooding_seed_bind_group_layout.clone(),
                    jump_flooding_uniform.clone(),
                ],
                push_constant_ranges: vec![],
                shader,
                shader_defs: vec![],
                entry_point: Cow::from("jump"),
            });

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/recompute_levelset/recompute_sdf.wgsl");

        let sdf_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("RecomputeLevelset sdf_pipeline")),
            layout: vec![
                jump_flooding_seed_bind_group_layout.clone(),
                levelset_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("sdf"),
        });

        Self {
            initialize_pipeline,
            jump_flood_pipeline,
            sdf_pipeline,
        }
    }
}
