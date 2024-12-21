use std::borrow::Cow;

use bevy::{
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
        texture::{FallbackImage, GpuImage},
    },
};

use super::{grid_label::GridLabelMaterial, uniform::SimulationUniform};

pub fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<AdvectionPipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    textures: Res<AdvectionMaterial>,
    render_device: Res<RenderDevice>,
    fallback_image: Res<FallbackImage>,
) {
    let bind_group = textures
        .as_bind_group(
            &pipeline.bind_group_layout,
            &render_device,
            &gpu_images,
            &fallback_image,
        )
        .unwrap()
        .bind_group;

    commands.insert_resource(AdvectionBindGroup(bind_group));
}

#[derive(Resource, Clone, ExtractResource, AsBindGroup)]
pub struct AdvectionMaterial {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub u_in: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub u_out: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadWrite)]
    pub v_in: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = ReadWrite)]
    pub v_out: Handle<Image>,
}

#[derive(Resource)]
pub struct AdvectionBindGroup(pub BindGroup);

#[derive(Resource)]
pub struct AdvectionPipeline {
    bind_group_layout: BindGroupLayout,
    pub init_pipeline: CachedComputePipelineId,
    pub pipeline: CachedComputePipelineId,
    pub swap_pipeline: CachedComputePipelineId,
}

impl FromWorld for AdvectionPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = AdvectionMaterial::bind_group_layout(render_device);
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
                bind_group_layout.clone(),
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
                bind_group_layout.clone(),
                uniform_bind_group_layout.clone(),
                grid_label_bind_group_layout.clone(),
            ],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("initialize"),
        });

        let swap_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("swap"),
        });

        Self {
            bind_group_layout,
            init_pipeline,
            pipeline,
            swap_pipeline,
        }
    }
}
