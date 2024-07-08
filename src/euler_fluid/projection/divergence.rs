use std::borrow::Cow;

use bevy::{
    asset::{AssetServer, Handle},
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayout, CachedComputePipelineId,
            ComputePipelineDescriptor, PipelineCache,
        },
        renderer::RenderDevice,
        texture::{FallbackImage, Image},
    },
};

use crate::euler_fluid::grid_label::GridLabelMaterial;

#[derive(Resource, ExtractResource, AsBindGroup, Clone)]
pub struct DivergenceMaterial {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub u: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub v: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadWrite)]
    pub div: Handle<Image>,
}

#[derive(Resource)]
pub struct DivergenceBindGroup(pub BindGroup);

#[derive(Resource)]
pub struct DivergencePipeline {
    bind_group_layout: BindGroupLayout,
    pub pipeline: CachedComputePipelineId,
}

impl FromWorld for DivergencePipeline {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = DivergenceMaterial::bind_group_layout(render_device);
        let grid_label_bind_group_layout = GridLabelMaterial::bind_group_layout(render_device);
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/divergence.wgsl");

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![
                bind_group_layout.clone(),
                grid_label_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("divergence"),
        });

        Self {
            bind_group_layout,
            pipeline,
        }
    }
}

pub fn prepare_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<Image>>,
    pipeline: Res<DivergencePipeline>,
    material: Res<DivergenceMaterial>,
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

    commands.insert_resource(DivergenceBindGroup(bind_group));
}
