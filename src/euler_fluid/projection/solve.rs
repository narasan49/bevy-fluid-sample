use std::borrow::Cow;

use bevy::{
    asset::{AssetServer, Handle},
    prelude::*,
    render::{
        extract_component::ComponentUniforms,
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

use crate::euler_fluid::uniform::{SimulationUniform, SimulationUniformBindGroup};

#[derive(Resource, ExtractResource, AsBindGroup, Clone)]
pub struct SolvePressureMaterial {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub u_in: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub v_in: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadWrite)]
    pub u_out: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = ReadWrite)]
    pub v_out: Handle<Image>,
    #[storage_texture(4, image_format = R32Float, access = ReadWrite)]
    pub p: Handle<Image>,
}

#[derive(Resource)]
pub struct SolvePressureBindGroup(pub BindGroup);

#[derive(Resource)]
pub struct SolvePressurePipeline {
    bind_group_layout: BindGroupLayout,
    pub pipeline: CachedComputePipelineId,
}

impl FromWorld for SolvePressurePipeline {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = SolvePressureMaterial::bind_group_layout(render_device);
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
            layout: vec![bind_group_layout.clone(), uniform_bind_group_layout.clone()],
            push_constant_ranges: vec![],
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("solve_pressure"),
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
    pipeline: Res<SolvePressurePipeline>,
    material: Res<SolvePressureMaterial>,
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

    commands.insert_resource(SolvePressureBindGroup(bind_group));
}
