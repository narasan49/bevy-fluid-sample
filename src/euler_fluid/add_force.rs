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

use super::uniform::SimulationUniform;

pub fn prepare_bind_group(
    mut commands: Commands,
    gpu_images: Res<RenderAssets<GpuImage>>,
    pipeline: Res<AddForcePipeline>,
    textures: Res<AddForceMaterial>,
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

    commands.insert_resource(AddForceBindGroup(bind_group));
}

#[derive(Resource, Clone, ExtractResource, AsBindGroup)]
pub struct AddForceMaterial {
    #[storage(0, read_only, visibility(compute))]
    pub force: Vec<Vec2>,
    #[storage(1, read_only, visibility(compute))]
    pub position: Vec<Vec2>,
    #[storage_texture(2, image_format = R32Float, access = ReadWrite)]
    pub u: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = ReadWrite)]
    pub v: Handle<Image>,
}

#[derive(Resource)]
pub struct AddForceBindGroup(pub BindGroup);

#[derive(Resource)]
pub struct AddForcePipeline {
    bind_group_layout: BindGroupLayout,
    pub pipeline: CachedComputePipelineId,
}

impl FromWorld for AddForcePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = AddForceMaterial::bind_group_layout(render_device);
        let uniform_bind_group_layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<SimulationUniform>(false),
            ),
        );

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/add_force.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![bind_group_layout.clone(), uniform_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("add_force"),
        });

        Self {
            bind_group_layout,
            pipeline,
        }
    }
}
