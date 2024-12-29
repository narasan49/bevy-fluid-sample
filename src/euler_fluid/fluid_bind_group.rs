use std::borrow::Cow;

use bevy::render::extract_component::ExtractComponent;
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

use super::definition::{
    GridCenterTextures, LocalForces, Obstacles, SimulationUniform, VelocityTextures,
};

#[derive(Resource)]
pub(crate) struct FluidPipelines {
    pub initialize_velocity_pipeline: CachedComputePipelineId,
    pub initialize_grid_center_pipeline: CachedComputePipelineId,
    pub update_grid_label_pipeline: CachedComputePipelineId,
    pub advection_pipeline: CachedComputePipelineId,
    pub add_force_pipeline: CachedComputePipelineId,
    pub divergence_pipeline: CachedComputePipelineId,
    pub jacobi_iteration_pipeline: CachedComputePipelineId,
    pub jacobi_iteration_reverse_pipeline: CachedComputePipelineId,
    pub solve_velocity_pipeline: CachedComputePipelineId,
    velocity_bind_group_layout: BindGroupLayout,
    grid_center_bind_group_layout: BindGroupLayout,
    local_forces_bind_group_layout: BindGroupLayout,
    uniform_bind_group_layout: BindGroupLayout,
    obstacles_bind_group_layout: BindGroupLayout,
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
        let obstacles_bind_group_layout = Obstacles::bind_group_layout(render_device);

        let initialize_velocity_shader = asset_server.load("shaders/initialize_velocity.wgsl");
        let initialize_velocity_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue InitializeVelocityPipeline")),
                layout: vec![velocity_bind_group_layout.clone()],
                push_constant_ranges: vec![],
                shader: initialize_velocity_shader,
                shader_defs: vec![],
                entry_point: Cow::from("initialize_velocity"),
            });

        let initialize_grid_center_shader =
            asset_server.load("shaders/initialize_grid_center.wgsl");
        let initialize_grid_center_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue InitializeGridCenterPipeline")),
                layout: vec![grid_center_bind_group_layout.clone()],
                push_constant_ranges: vec![],
                shader: initialize_grid_center_shader,
                shader_defs: vec![],
                entry_point: Cow::from("initialize_grid_center"),
            });

        let update_grid_label_shader = asset_server.load("shaders/update_grid_label.wgsl");
        let update_grid_label_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue UpdateGridLabelPipeline")),
                layout: vec![
                    velocity_bind_group_layout.clone(),
                    grid_center_bind_group_layout.clone(),
                    obstacles_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: update_grid_label_shader,
                shader_defs: vec![],
                entry_point: Cow::from("update_grid_label"),
            });

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

        let divergence_shader = asset_server.load("shaders/divergence.wgsl");
        let divergence_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue DivergencePipeline")),
                layout: vec![
                    velocity_bind_group_layout.clone(),
                    grid_center_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: divergence_shader,
                shader_defs: vec![],
                entry_point: Cow::from("divergence"),
            });

        let jacobi_iteration_shader = asset_server.load("shaders/jacobi_iteration.wgsl");
        let jacobi_iteration_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue JacobiIterationPipeline")),
                layout: vec![
                    uniform_bind_group_layout.clone(),
                    grid_center_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: jacobi_iteration_shader.clone(),
                shader_defs: vec![],
                entry_point: Cow::from("jacobi_iteration"),
            });

        let jacobi_iteration_reverse_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue JacobiIterationReversePipeline")),
                layout: vec![
                    uniform_bind_group_layout.clone(),
                    grid_center_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: jacobi_iteration_shader,
                shader_defs: vec![],
                entry_point: Cow::from("jacobi_iteration_reverse"),
            });

        let solve_velocity_shader = asset_server.load("shaders/solve_velocity.wgsl");
        let solve_velocity_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue SolveVelocityPipeline")),
                layout: vec![
                    velocity_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                    grid_center_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: solve_velocity_shader,
                shader_defs: vec![],
                entry_point: Cow::from("solve_velocity"),
            });

        Self {
            initialize_velocity_pipeline,
            initialize_grid_center_pipeline,
            update_grid_label_pipeline,
            advection_pipeline,
            add_force_pipeline,
            divergence_pipeline,
            jacobi_iteration_pipeline,
            jacobi_iteration_reverse_pipeline,
            solve_velocity_pipeline,
            velocity_bind_group_layout,
            grid_center_bind_group_layout,
            local_forces_bind_group_layout,
            uniform_bind_group_layout,
            obstacles_bind_group_layout,
        }
    }
}

#[derive(Component, Clone, ExtractComponent)]
pub(crate) struct FluidBindGroups {
    pub velocity_bind_group: BindGroup,
    pub grid_center_bind_group: BindGroup,
    pub local_forces_bind_group: BindGroup,
    pub uniform_bind_group: BindGroup,
    pub uniform_index: u32,
}

#[derive(Resource)]
pub(crate) struct FluidBindGroupResources {
    pub obstacles_bind_group: BindGroup,
}

pub(super) fn prepare_fluid_bind_groups(
    mut commands: Commands,
    pipelines: Res<FluidPipelines>,
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
            &pipelines.uniform_bind_group_layout,
            &BindGroupEntries::single(simulation_uniform),
        );

        let velocity_bind_group = advection_textures
            .as_bind_group(
                &pipelines.velocity_bind_group_layout,
                &render_device,
                &gpu_images,
                &fallback_image,
            )
            .unwrap()
            .bind_group;

        let grid_center_bind_group = add_force_textures
            .as_bind_group(
                &pipelines.grid_center_bind_group_layout,
                &render_device,
                &gpu_images,
                &fallback_image,
            )
            .unwrap()
            .bind_group;

        let local_forces_bind_group = local_forces
            .as_bind_group(
                &pipelines.local_forces_bind_group_layout,
                &render_device,
                &gpu_images,
                &fallback_image,
            )
            .unwrap()
            .bind_group;

        commands.entity(entity).insert(FluidBindGroups {
            velocity_bind_group,
            grid_center_bind_group,
            local_forces_bind_group,
            uniform_bind_group,
            uniform_index: simulation_uniform_index.index(),
        });
    }
}

pub(super) fn prepare_fluid_bind_group_for_resources(
    mut commands: Commands,
    pilelines: Res<FluidPipelines>,
    obstacles: Res<Obstacles>,
    render_device: Res<RenderDevice>,
    fallback_image: Res<FallbackImage>,
    gpu_images: Res<RenderAssets<GpuImage>>,
) {
    let obstacles_bind_group = obstacles
        .as_bind_group(
            &pilelines.obstacles_bind_group_layout,
            &render_device,
            &gpu_images,
            &fallback_image,
        )
        .unwrap()
        .bind_group;
    commands.insert_resource(FluidBindGroupResources {
        obstacles_bind_group,
    });
}
