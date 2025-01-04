use std::borrow::Cow;

use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::UniformBuffer;
use bevy::render::renderer::RenderQueue;
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
    DivergenceTextures, FluidSettings, JumpFloodingSeedsTextures, JumpFloodingUniform,
    JumpFloodingUniformBuffer, LevelsetTextures, LocalForces, Obstacles, PressureTextures,
    SimulationUniform, VelocityTextures,
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
    pub recompute_levelset_initialization_pipeline: CachedComputePipelineId,
    pub recompute_levelset_iteration_pipeline: CachedComputePipelineId,
    pub recompute_levelset_solve_pipeline: CachedComputePipelineId,
    pub advect_levelset_pipeline: CachedComputePipelineId,
    velocity_bind_group_layout: BindGroupLayout,
    pressure_bind_group_layout: BindGroupLayout,
    divergence_bind_group_layout: BindGroupLayout,
    levelset_bind_group_layout: BindGroupLayout,
    local_forces_bind_group_layout: BindGroupLayout,
    uniform_bind_group_layout: BindGroupLayout,
    obstacles_bind_group_layout: BindGroupLayout,
    jump_flooding_seeds_bind_group_layout: BindGroupLayout,
    jump_flooding_uniform_bind_group_layout: BindGroupLayout,
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
        let pressure_bind_group_layout = PressureTextures::bind_group_layout(render_device);
        let divergence_bind_group_layout = DivergenceTextures::bind_group_layout(render_device);
        let levelset_bind_group_layout = LevelsetTextures::bind_group_layout(render_device);
        let obstacles_bind_group_layout = Obstacles::bind_group_layout(render_device);
        let jump_flooding_seeds_bind_group_layout =
            JumpFloodingSeedsTextures::bind_group_layout(render_device);
        let jump_flooding_uniform_bind_group_layout = render_device.create_bind_group_layout(
            Some("Create JumpFloodingUniformBindGroupLayout"),
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<JumpFloodingUniform>(false),
            ),
        );

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
                layout: vec![
                    levelset_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                ],
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
                    levelset_bind_group_layout.clone(),
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
                levelset_bind_group_layout.clone(),
                uniform_bind_group_layout.clone(),
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
                    divergence_bind_group_layout.clone(),
                    levelset_bind_group_layout.clone(),
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
                    pressure_bind_group_layout.clone(),
                    divergence_bind_group_layout.clone(),
                    levelset_bind_group_layout.clone(),
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
                    pressure_bind_group_layout.clone(),
                    divergence_bind_group_layout.clone(),
                    levelset_bind_group_layout.clone(),
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
                    pressure_bind_group_layout.clone(),
                    levelset_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: solve_velocity_shader,
                shader_defs: vec![],
                entry_point: Cow::from("solve_velocity"),
            });

        let recompute_levelset_initialization_shader =
            asset_server.load("shaders/recompute_levelset/initialize.wgsl");
        let recompute_levelset_initialization_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue RecomputeLevelsetInitializationPipeline")),
                layout: vec![
                    levelset_bind_group_layout.clone(),
                    jump_flooding_seeds_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: recompute_levelset_initialization_shader,
                shader_defs: vec![],
                entry_point: Cow::from("initialize"),
            });

        let recompute_levelset_iteration_shader =
            asset_server.load("shaders/recompute_levelset/iterate.wgsl");
        let recompute_levelset_iteration_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue RecomputeLevelsetIteratePipeline")),
                layout: vec![
                    jump_flooding_seeds_bind_group_layout.clone(),
                    jump_flooding_uniform_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: recompute_levelset_iteration_shader,
                shader_defs: vec![],
                entry_point: Cow::from("iterate"),
            });

        let recompute_levelset_solve_shader =
            asset_server.load("shaders/recompute_levelset/calculate_sdf.wgsl");
        let recompute_levelset_solve_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue RecomputeLevelsetSolvePipeline")),
                layout: vec![
                    levelset_bind_group_layout.clone(),
                    jump_flooding_seeds_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: recompute_levelset_solve_shader,
                shader_defs: vec![],
                entry_point: Cow::from("calculate_sdf"),
            });

        let advect_levelset_shader = asset_server.load("shaders/advect_levelset.wgsl");
        let advect_levelset_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("Queue AdvectLevelsetPipeline")),
                layout: vec![
                    velocity_bind_group_layout.clone(),
                    levelset_bind_group_layout.clone(),
                    uniform_bind_group_layout.clone(),
                ],
                push_constant_ranges: vec![],
                shader: advect_levelset_shader,
                shader_defs: vec![],
                entry_point: Cow::from("advect_levelset"),
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
            recompute_levelset_initialization_pipeline,
            recompute_levelset_iteration_pipeline,
            recompute_levelset_solve_pipeline,
            advect_levelset_pipeline,
            velocity_bind_group_layout,
            pressure_bind_group_layout,
            divergence_bind_group_layout,
            levelset_bind_group_layout,
            local_forces_bind_group_layout,
            uniform_bind_group_layout,
            obstacles_bind_group_layout,
            jump_flooding_uniform_bind_group_layout,
            jump_flooding_seeds_bind_group_layout,
        }
    }
}

#[derive(Component, Clone, ExtractComponent)]
pub(crate) struct FluidBindGroups {
    pub velocity_bind_group: BindGroup,
    pub pressure_bind_group: BindGroup,
    pub divergence_bind_group: BindGroup,
    pub local_forces_bind_group: BindGroup,
    pub levelset_bind_group: BindGroup,
    pub jump_flooding_seeds_bind_group: BindGroup,
    pub uniform_bind_group: BindGroup,
    pub uniform_index: u32,
}

#[derive(Resource)]
pub(crate) struct FluidBindGroupResources {
    pub obstacles_bind_group: BindGroup,
}

/// Different from [`FluidBindGroups`], [`DynamicUniformIndex`] will not be used.
/// Here, several bindings for jump flooding steps for each component.
/// However, only one index can be used per component on [`DynamicUniformIndex`].
/// Therefore, array of bind groups per component is adopted here.
#[derive(Component)]
pub(crate) struct JumpFloodingUniformBindGroups {
    pub jump_flooding_step_bind_groups: Box<[BindGroup]>,
    // pub uniform_index: u32,
}

pub(super) fn prepare_resource_recompute_levelset(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    query: Query<(Entity, &FluidSettings)>,
) {
    for (entity, settings) in &query {
        // steps for jump flooding algorithm: 1, 2, ..., 2^k, where: 2^k < max(size.0, size.1) <= 2^(k+1)
        let max_power =
            ((settings.size.0.max(settings.size.1) as f32).log2() - 1.0).floor() as usize;
        let mut step = 2_u32.pow((max_power + 1) as u32);
        let mut jump_flooding_buffer =
            Vec::<UniformBuffer<JumpFloodingUniform>>::with_capacity(max_power + 1);
        for _ in 0..max_power + 1 {
            step /= 2;
            jump_flooding_buffer.push(UniformBuffer::from(JumpFloodingUniform { step }));
        }
        for buffer in &mut jump_flooding_buffer {
            buffer.write_buffer(&render_device, &render_queue);
        }

        commands.entity(entity).insert(JumpFloodingUniformBuffer {
            buffer: jump_flooding_buffer,
        });
    }
}

pub(super) fn prepare_fluid_bind_groups(
    mut commands: Commands,
    pipelines: Res<FluidPipelines>,
    simulation_uniform: Res<ComponentUniforms<SimulationUniform>>,
    query: Query<(
        Entity,
        &VelocityTextures,
        &PressureTextures,
        &DivergenceTextures,
        &LevelsetTextures,
        &LocalForces,
        &DynamicUniformIndex<SimulationUniform>,
        &JumpFloodingSeedsTextures,
        &JumpFloodingUniformBuffer,
    )>,
    render_device: Res<RenderDevice>,
    fallback_image: Res<FallbackImage>,
    gpu_images: Res<RenderAssets<GpuImage>>,
) {
    for (
        entity,
        velocity_textures,
        pressure_textures,
        divergence_textures,
        levelset_textures,
        local_forces,
        simulation_uniform_index,
        jump_flooding_seeds_textures,
        jump_flooding_uniform_buffer,
    ) in &query
    {
        let simulation_uniform = simulation_uniform.uniforms();
        let uniform_bind_group = render_device.create_bind_group(
            "Simulation Uniform BindGroup",
            &pipelines.uniform_bind_group_layout,
            &BindGroupEntries::single(simulation_uniform),
        );

        let velocity_bind_group = velocity_textures
            .as_bind_group(
                &pipelines.velocity_bind_group_layout,
                &render_device,
                &gpu_images,
                &fallback_image,
            )
            .unwrap()
            .bind_group;

        let pressure_bind_group = pressure_textures
            .as_bind_group(
                &pipelines.pressure_bind_group_layout,
                &render_device,
                &gpu_images,
                &fallback_image,
            )
            .unwrap()
            .bind_group;

        let divergence_bind_group = divergence_textures
            .as_bind_group(
                &pipelines.divergence_bind_group_layout,
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

        let mut jump_flooding_step_bind_groups =
            Vec::with_capacity(jump_flooding_uniform_buffer.buffer.len());
        for buffer in &jump_flooding_uniform_buffer.buffer {
            jump_flooding_step_bind_groups.push(render_device.create_bind_group(
                Some("Create JumpFloodingStepBindGroup"),
                &pipelines.jump_flooding_uniform_bind_group_layout,
                &BindGroupEntries::single(buffer.binding().unwrap()),
            ));
        }

        let levelset_bind_group = levelset_textures
            .as_bind_group(
                &pipelines.levelset_bind_group_layout,
                &render_device,
                &gpu_images,
                &fallback_image,
            )
            .unwrap()
            .bind_group;

        let jump_flooding_seeds_bind_group = jump_flooding_seeds_textures
            .as_bind_group(
                &pipelines.jump_flooding_seeds_bind_group_layout,
                &render_device,
                &gpu_images,
                &fallback_image,
            )
            .unwrap()
            .bind_group;

        commands.entity(entity).insert((
            FluidBindGroups {
                velocity_bind_group,
                pressure_bind_group,
                divergence_bind_group,
                local_forces_bind_group,
                levelset_bind_group,
                jump_flooding_seeds_bind_group,
                uniform_bind_group,
                uniform_index: simulation_uniform_index.index(),
            },
            JumpFloodingUniformBindGroups {
                jump_flooding_step_bind_groups: jump_flooding_step_bind_groups.into_boxed_slice(),
            },
        ));
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
