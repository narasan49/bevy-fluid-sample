use std::borrow::Cow;

use bevy::{
    asset::{AssetServer, Handle},
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_asset::RenderAssets,
        render_resource::{
            binding_types::uniform_buffer, AsBindGroup, BindGroup, BindGroupLayout,
            BindGroupLayoutEntries, CachedComputePipelineId, ComputePipelineDescriptor,
            PipelineCache, ShaderStages,
        },
        renderer::RenderDevice,
        texture::{FallbackImage, Image},
    },
};

use crate::euler_fluid::{grid_label::GridLabelMaterial, uniform::SimulationUniform};

#[derive(Resource, ExtractResource, AsBindGroup, Clone)]
pub struct JacobiMaterial {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub div: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub p0: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadWrite)]
    pub p1: Handle<Image>,
}

#[derive(Resource)]
pub struct JacobiBindGroup(pub BindGroup);

#[derive(Resource)]
pub struct JacobiPipeline {
    pub bind_group_layout: BindGroupLayout,
    pub pipeline: CachedComputePipelineId,
    pub swap_pipeline: CachedComputePipelineId,
}

impl FromWorld for JacobiPipeline {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let render_device = world.resource();
        let bind_group_layout = JacobiMaterial::bind_group_layout(render_device);
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
                bind_group_layout.clone(),
                uniform_bind_group_layout.clone(),
                grid_label_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("jacobi_iteration"),
        });

        let swap_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![bind_group_layout.clone()],
            push_constant_ranges: vec![],
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("swap_buffers"),
        });

        Self {
            bind_group_layout,
            pipeline,
            swap_pipeline,
        }
    }
}

pub fn prepare_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<Image>>,
    pipeline: Res<JacobiPipeline>,
    material: Res<JacobiMaterial>,
    fallback_image: Res<FallbackImage>,
) {
    let bind_group = material
        .as_bind_group(
            &pipeline.bind_group_layout,
            &render_device,
            &gpu_images,
            &fallback_image,
        )
        .unwrap()
        .bind_group;

    commands.insert_resource(JacobiBindGroup(bind_group));
}
