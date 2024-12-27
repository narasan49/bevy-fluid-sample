use std::borrow::Cow;

use bevy::{
    prelude::*,
    render::{
        extract_component::{ComponentUniforms, DynamicUniformIndex},
        render_asset::RenderAssets,
        render_resource::{
            binding_types::uniform_buffer, AsBindGroup, BindGroup, BindGroupEntries,
            BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId,
            ComputePipelineDescriptor, PipelineCache, ShaderStages,
        },
        renderer::RenderDevice,
        texture::{FallbackImage, GpuImage},
    },
};

use super::{
    definition::{GridCenterTextures, LocalForces, VelocityTextures},
    uniform::SimulationUniform,
};

#[derive(Resource)]
pub struct FluidPipelines {
    pub advection_pipeline: CachedComputePipelineId,
    pub add_force_pipeline: CachedComputePipelineId,
    velocity_bind_group_layout: BindGroupLayout,
    grid_center_bind_group_layout: BindGroupLayout,
    local_forces_bind_group_layout: BindGroupLayout,
    uniform_bind_group_layout: BindGroupLayout,
}

impl FromWorld for FluidPipelines {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let asset_server = world.resource::<AssetServer>();

        let uniform_bind_group_layout = render_device.create_bind_group_layout(
            Some("Create uniform bind group layout"),
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<SimulationUniform>(true),
            ),
        );
        let velocity_bind_group_layout = VelocityTextures::bind_group_layout(render_device);
        let local_forces_bind_group_layout = LocalForces::bind_group_layout(render_device);
        let grid_center_bind_group_layout = GridCenterTextures::bind_group_layout(render_device);

        let advection_shader = asset_server.load("shaders/advection.wgsl");
        let advection_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("Queue AdvectionPipeline")),
            layout: vec![
                velocity_bind_group_layout.clone(),
                uniform_bind_group_layout.clone(),
                grid_center_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: advection_shader,
            shader_defs: vec![],
            entry_point: Cow::from("advection"),
        });

        let add_force_shader = asset_server.load("shaders/add_force.wgsl");
        let add_force_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("Queue AddForcePipeline")),
            layout: vec![
                velocity_bind_group_layout.clone(),
                uniform_bind_group_layout.clone(),
                local_forces_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: add_force_shader,
            shader_defs: vec![],
            entry_point: Cow::from("add_force"),
        });

        Self {
            advection_pipeline,
            add_force_pipeline,
            velocity_bind_group_layout,
            grid_center_bind_group_layout,
            local_forces_bind_group_layout,
            uniform_bind_group_layout,
        }
    }
}

#[derive(Component)]
pub struct FluidBindGroups {
    pub velocity_bind_group: BindGroup,
    pub grid_center_bind_group: BindGroup,
    pub local_forces_bind_group: BindGroup,
    pub uniform_bind_group: BindGroup,
    pub uniform_index: u32,
}

pub fn prepare_fluid_bind_groups(
    mut commands: Commands,
    pilelines: Res<FluidPipelines>,
    simulation_uniform: Res<ComponentUniforms<SimulationUniform>>,
    query: Query<(
        Entity,
        &VelocityTextures,
        &GridCenterTextures,
        &LocalForces,
        &DynamicUniformIndex<SimulationUniform>,
    )>,
    render_device: Res<RenderDevice>,
    fallback_image: Res<FallbackImage>,
    gpu_images: Res<RenderAssets<GpuImage>>,
) {
    for (entity, advection_textures, add_force_textures, local_forces, simulation_uniform_index) in
        &query
    {
        let simulation_uniform = simulation_uniform.uniforms();
        let uniform_bind_group = render_device.create_bind_group(
            "Simulation Uniform BindGroup",
            &pilelines.uniform_bind_group_layout,
            &BindGroupEntries::single(simulation_uniform),
        );

        let velocity_bind_group = advection_textures
            .as_bind_group(
                &pilelines.velocity_bind_group_layout,
                &render_device,
                &gpu_images,
                &fallback_image,
            )
            .unwrap()
            .bind_group;

        let grid_center_bind_group = add_force_textures
            .as_bind_group(
                &pilelines.grid_center_bind_group_layout,
                &render_device,
                &gpu_images,
                &fallback_image,
            )
            .unwrap()
            .bind_group;

        let local_forces_bind_group = local_forces
            .as_bind_group(
                &pilelines.local_forces_bind_group_layout,
                &render_device,
                &gpu_images,
                &fallback_image,
            )
            .unwrap()
            .bind_group;

        info!("Inserting FluidBindGroups into entity ({:?}).", entity);
        commands.entity(entity).insert(FluidBindGroups {
            velocity_bind_group,
            grid_center_bind_group,
            local_forces_bind_group,
            uniform_bind_group,
            uniform_index: simulation_uniform_index.index(),
        });
    }
}
