use std::borrow::Cow;

use bevy::{
    prelude::*,
    render::{
        extract_component::{ComponentUniforms, DynamicUniformIndex},
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

use super::{advection::AdvectionTextures, uniform::SimulationUniform};

#[derive(Resource)]
pub struct FluidPipelines {
    pub advection_pipeline: CachedComputePipelineId,
    advection_bind_group_layout: BindGroupLayout,
}

impl FromWorld for FluidPipelines {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let uniform_bind_group_layout = render_device.create_bind_group_layout(
            Some("Create uniform bind group layout"),
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<SimulationUniform>(true),
            ),
        );
        let advection_bind_group_layout = AdvectionTextures::bind_group_layout(render_device);

        let advection_shader = world.resource::<AssetServer>().load("asset/advection.wgsl");
        let advection_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("Queue AdvectionPipeline")),
            layout: vec![
                advection_bind_group_layout.clone(),
                uniform_bind_group_layout,
            ],
            push_constant_ranges: vec![],
            shader: advection_shader,
            shader_defs: vec![],
            entry_point: Cow::from("advection"),
        });

        Self {
            advection_pipeline,
            advection_bind_group_layout,
        }
    }
}

#[derive(Component)]
pub struct FluidBindGroups {
    pub advection_bind_group: BindGroup,
    pub uniform_index: u32,
}

#[derive(Resource, Clone, ExtractResource, AsBindGroup)]
pub struct LocalForceFieldsMaterial {
    #[storage(0, read_only, visibility(compute))]
    pub force: Vec<Vec2>,
    #[storage(1, read_only, visibility(compute))]
    pub position: Vec<Vec2>,
}

pub fn prepare_fluid_bind_groups(
    mut commands: Commands,
    pilelines: Res<FluidPipelines>,
    simulation_uniform: Res<ComponentUniforms<SimulationUniform>>,
    query: Query<(
        Entity,
        &AdvectionTextures,
        &DynamicUniformIndex<SimulationUniform>,
    )>,
    render_device: Res<RenderDevice>,
    fallback_image: Res<FallbackImage>,
    gpu_images: Res<RenderAssets<GpuImage>>,
) {
    for (entity, advection_textures, simulation_uniform_index) in &query {
        let simulation_uniform = simulation_uniform.uniforms();
        let advection_bind_group = advection_textures
            .as_bind_group(
                &pilelines.advection_bind_group_layout,
                &render_device,
                &gpu_images,
                &fallback_image,
            )
            .unwrap()
            .bind_group;

        commands.entity(entity).insert(FluidBindGroups {
            advection_bind_group,
            uniform_index: simulation_uniform_index.index(),
        });
    }
}
