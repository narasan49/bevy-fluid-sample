pub mod add_force;
pub mod advection;
pub mod fluid_material;
pub mod geometry;
pub mod grid_label;
pub mod materials;
pub mod pipelines;
pub mod projection;
pub mod uniform;

use add_force::AddForcePipeline;
use advection::AdvectionPipeline;
use bevy::{
    asset::load_internal_asset,
    math::vec2,
    prelude::*,
    render::{
        extract_component::{ComponentUniforms, ExtractComponentPlugin, UniformComponentPlugin},
        extract_resource::ExtractResourcePlugin,
        graph::CameraDriverLabel,
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{binding_types::uniform_buffer, *},
        renderer::RenderDevice,
        Render, RenderApp, RenderSet,
    },
    sprite::Material2dPlugin,
};
use fluid_material::VelocityMaterial;
use geometry::{CircleCollectionBindGroup, CircleCollectionMaterial, CrircleUniform, Velocity};
use grid_label::{GridLabelBindGroup, GridLabelMaterial, GridLabelPipeline};
use materials::{
    divergence::{DivergenceBindGroup, DivergenceBindGroupLayout, DivergenceMaterial},
    jfa_seeds::{
        JumpFloodingSeedBindGroupLayout, JumpFloodingSeedsMaterial, JumpFoodingSeedsBindGroup,
    },
    jump_flooding_uniform::{JumpFloodPlugin, JumpFloodingUniformBindGroups},
    levelset::{LevelSetBindGroup, LevelSetBindGroupLayout, LevelSetMaterial},
    local_force::{LocalForceBindGroup, LocalForceBindGroupLayout, LocalForceMaterial},
    prepare_bind_group::PrepareBindGroup,
    pressure::{
        IntermediatePressureBindGroupLayout, IntermediatePressureMaterial, PressureBindGroup,
        PressureBindGroupLayout, PressureMaterial,
    },
    staggered_velocity::{
        IntermediateVelocityBindGroup, IntermediateVelocityBindGroupLayout,
        StaggeredIntermediateVelocityMaterial, StaggeredVelocityMaterial, VelocityBindGroup,
        VelocityBindGroupLayout,
    },
};
use pipelines::{
    advect_levelset::AdvectLevelsetPipeline, recompute_levelset::RecomputeLevelsetPipeline,
};
use projection::{
    divergence::DivergencePipeline, jacobi_iteration::JacobiPipeline, solve::SolvePressurePipeline,
};
use uniform::{SimulationUniform, SimulationUniformBindGroup};

use crate::texture::NewTexture;

use self::fluid_material::FluidMaterial;

const SIZE: (u32, u32) = (512, 512);
const SIZE_U: (u32, u32) = (SIZE.0 + 1, SIZE.1);
const SIZE_V: (u32, u32) = (SIZE.0, SIZE.1 + 1);
const WORKGROUP_SIZE: u32 = 8;

const FLUID_UNIFORM_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x8B9323522322463BA8CF530771C532EF);

const COORDINATE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x9F8E2E5B1E5F40C096C31175C285BF11);

pub struct FluidPlugin;

impl Plugin for FluidPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<StaggeredVelocityMaterial>::default())
            .add_plugins(ExtractResourcePlugin::<StaggeredIntermediateVelocityMaterial>::default())
            .add_plugins(ExtractResourcePlugin::<LocalForceMaterial>::default())
            .add_plugins(ExtractResourcePlugin::<DivergenceMaterial>::default())
            .add_plugins(ExtractResourcePlugin::<PressureMaterial>::default())
            .add_plugins(ExtractResourcePlugin::<IntermediatePressureMaterial>::default())
            .add_plugins(ExtractResourcePlugin::<GridLabelMaterial>::default())
            .add_plugins(ExtractResourcePlugin::<LevelSetMaterial>::default())
            .add_plugins(ExtractResourcePlugin::<CircleCollectionMaterial>::default())
            .add_plugins(ExtractComponentPlugin::<SimulationUniform>::default())
            .add_plugins(UniformComponentPlugin::<SimulationUniform>::default())
            .add_plugins(ExtractResourcePlugin::<JumpFloodingSeedsMaterial>::default())
            .add_plugins(MaterialPlugin::<FluidMaterial>::default())
            .add_plugins(Material2dPlugin::<FluidMaterial>::default())
            .add_plugins(MaterialPlugin::<VelocityMaterial>::default())
            .add_plugins(Material2dPlugin::<VelocityMaterial>::default())
            .add_plugins(JumpFloodPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, update_geometry);

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(
                Render,
                prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
            )
            .add_systems(
                Render,
                LocalForceBindGroupLayout::prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
            )
            .add_systems(
                Render,
                VelocityBindGroupLayout::prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
            )
            .add_systems(
                Render,
                IntermediateVelocityBindGroupLayout::prepare_bind_group
                    .in_set(RenderSet::PrepareBindGroups),
            )
            .add_systems(
                Render,
                DivergenceBindGroupLayout::prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
            )
            .add_systems(
                Render,
                PressureBindGroupLayout::prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
            )
            .add_systems(
                Render,
                IntermediatePressureBindGroupLayout::prepare_bind_group
                    .in_set(RenderSet::PrepareBindGroups),
            )
            .add_systems(
                Render,
                LevelSetBindGroupLayout::prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
            )
            .add_systems(
                Render,
                grid_label::prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
            )
            .add_systems(
                Render,
                geometry::prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
            )
            .add_systems(
                Render,
                JumpFloodingSeedBindGroupLayout::prepare_bind_group
                    .in_set(RenderSet::PrepareBindGroups),
            );

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(FluidLabel, FluidNode::default());
        render_graph.add_node_edge(FluidLabel, CameraDriverLabel);

        load_internal_asset!(
            app,
            FLUID_UNIFORM_SHADER_HANDLE,
            "../assets/shaders/fluid_uniform.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            COORDINATE_SHADER_HANDLE,
            "../assets/shaders/coordinate.wgsl",
            Shader::from_wgsl
        )
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<VelocityBindGroupLayout>();
        render_app.init_resource::<IntermediateVelocityBindGroupLayout>();
        render_app.init_resource::<LocalForceBindGroupLayout>();
        render_app.init_resource::<DivergenceBindGroupLayout>();
        render_app.init_resource::<PressureBindGroupLayout>();
        render_app.init_resource::<IntermediatePressureBindGroupLayout>();
        render_app.init_resource::<LevelSetBindGroupLayout>();
        render_app.init_resource::<JumpFloodingSeedBindGroupLayout>();

        render_app.init_resource::<AdvectionPipeline>();
        render_app.init_resource::<AddForcePipeline>();
        render_app.init_resource::<SolvePressurePipeline>();
        render_app.init_resource::<DivergencePipeline>();
        render_app.init_resource::<JacobiPipeline>();
        render_app.init_resource::<GridLabelPipeline>();
        render_app.init_resource::<AdvectLevelsetPipeline>();
        render_app.init_resource::<RecomputeLevelsetPipeline>();
    }
}

fn prepare_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    uniform: Res<ComponentUniforms<SimulationUniform>>,
) {
    let uniform = uniform.uniforms().binding().unwrap();

    let bind_group_layout = render_device.create_bind_group_layout(
        None,
        &BindGroupLayoutEntries::single(
            ShaderStages::COMPUTE,
            uniform_buffer::<SimulationUniform>(false),
        ),
    );

    let uniform_bind_group = render_device.create_bind_group(
        None,
        &bind_group_layout,
        &BindGroupEntries::single(uniform),
    );

    commands.insert_resource(SimulationUniformBindGroup(uniform_bind_group));
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let u0 = images.new_texture_storage(SIZE_U, TextureFormat::R32Float);
    let u1 = images.new_texture_storage(SIZE_U, TextureFormat::R32Float);

    let v0 = images.new_texture_storage(SIZE_V, TextureFormat::R32Float);
    let v1 = images.new_texture_storage(SIZE_V, TextureFormat::R32Float);

    let div = images.new_texture_storage(SIZE, TextureFormat::R32Float);

    let p0 = images.new_texture_storage(SIZE, TextureFormat::R32Float);
    let p1 = images.new_texture_storage(SIZE, TextureFormat::R32Float);

    let grid_label = images.new_texture_storage(SIZE, TextureFormat::R32Uint);
    let u_solid = images.new_texture_storage(SIZE, TextureFormat::R32Float);
    let v_solid = images.new_texture_storage(SIZE, TextureFormat::R32Float);
    let levelset = images.new_texture_storage(SIZE, TextureFormat::R32Float);
    let seeds = images.new_texture_storage(SIZE, TextureFormat::Rg32Float);

    info!("inserting fluid resources.");
    commands.insert_resource(StaggeredVelocityMaterial { u: u0, v: v0 });

    commands.insert_resource(StaggeredIntermediateVelocityMaterial { u: u1, v: v1 });

    commands.insert_resource(LocalForceMaterial {
        force: vec![],
        position: vec![],
    });

    commands.insert_resource(DivergenceMaterial { div });

    commands.insert_resource(PressureMaterial { p: p0 });

    commands.insert_resource(IntermediatePressureMaterial { p: p1 });

    commands.insert_resource(GridLabelMaterial {
        grid_label,
        u_solid,
        v_solid,
    });

    commands.insert_resource(LevelSetMaterial { levelset });

    commands.insert_resource(CircleCollectionMaterial { circles: vec![] });

    commands.insert_resource(JumpFloodingSeedsMaterial { seeds });
}

fn update_geometry(
    query: Query<(&geometry::Circle, &Transform, &Velocity)>,
    mut geometries: ResMut<CircleCollectionMaterial>,
) {
    let circles = query
        .iter()
        .map(|(circle, transform, velocity)| {
            return CrircleUniform {
                r: circle.radius,
                position: transform.translation.xz(),
                velocity: vec2(velocity.u, velocity.v),
            };
        })
        .collect::<Vec<CrircleUniform>>();

    geometries.circles = circles;
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct FluidLabel;

enum FluidState {
    Loading,
    Init,
    Update,
}

struct FluidNode {
    state: FluidState,
}

impl Default for FluidNode {
    fn default() -> Self {
        Self {
            state: FluidState::Loading,
        }
    }
}

impl render_graph::Node for FluidNode {
    // update node state
    fn update(&mut self, world: &mut World) {
        let advection_pipeline = world.resource::<AdvectionPipeline>();
        let add_force_pipeline = world.resource::<AddForcePipeline>();
        let divergence_pipeline = world.resource::<DivergencePipeline>();
        let jacobi_pipeline = world.resource::<JacobiPipeline>();
        let solve_pipeline = world.resource::<SolvePressurePipeline>();
        let grid_label_pipeline = world.resource::<GridLabelPipeline>();
        let levelset_pipeline = world.resource::<AdvectLevelsetPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let recompute_levelset_pipeline = world.resource::<RecomputeLevelsetPipeline>();

        match self.state {
            FluidState::Loading => {
                let advection_pipeline =
                    pipeline_cache.get_compute_pipeline_state(advection_pipeline.init_pipeline);
                let grid_label_pipeline =
                    pipeline_cache.get_compute_pipeline_state(grid_label_pipeline.init_pipeline);
                let init_levelset_pipeline =
                    pipeline_cache.get_compute_pipeline_state(levelset_pipeline.init_pipeline);
                match (
                    advection_pipeline,
                    grid_label_pipeline,
                    init_levelset_pipeline,
                ) {
                    (
                        CachedPipelineState::Ok(_),
                        CachedPipelineState::Ok(_),
                        CachedPipelineState::Ok(_),
                    ) => {
                        self.state = FluidState::Init;
                    }
                    _ => {}
                }
            }
            FluidState::Init => {
                let advection_pipeline =
                    pipeline_cache.get_compute_pipeline_state(advection_pipeline.pipeline);
                let add_force_pipeline =
                    pipeline_cache.get_compute_pipeline_state(add_force_pipeline.pipeline);
                let jacobi_pipeline_state =
                    pipeline_cache.get_compute_pipeline_state(jacobi_pipeline.pipeline);
                let jacobi_swap_pipeline =
                    pipeline_cache.get_compute_pipeline_state(jacobi_pipeline.swap_pipeline);
                let solve_pipeline =
                    pipeline_cache.get_compute_pipeline_state(solve_pipeline.pipeline);
                let divergence_pipeline =
                    pipeline_cache.get_compute_pipeline_state(divergence_pipeline.pipeline);
                let grid_label_pipeline =
                    pipeline_cache.get_compute_pipeline_state(grid_label_pipeline.update_pipeline);
                let advect_levelset_pipeline =
                    pipeline_cache.get_compute_pipeline_state(levelset_pipeline.pipeline);
                let initialize_seeds_pipeline = pipeline_cache
                    .get_compute_pipeline_state(recompute_levelset_pipeline.initialize_pipeline);
                let jump_flooding_pipeline = pipeline_cache
                    .get_compute_pipeline_state(recompute_levelset_pipeline.jump_flood_pipeline);
                let update_sdf_pipeline = pipeline_cache
                    .get_compute_pipeline_state(recompute_levelset_pipeline.sdf_pipeline);
                match (
                    advection_pipeline,
                    add_force_pipeline,
                    jacobi_pipeline_state,
                    jacobi_swap_pipeline,
                    solve_pipeline,
                    divergence_pipeline,
                    grid_label_pipeline,
                    advect_levelset_pipeline,
                    initialize_seeds_pipeline,
                    jump_flooding_pipeline,
                    update_sdf_pipeline,
                ) {
                    (
                        CachedPipelineState::Ok(_),
                        CachedPipelineState::Ok(_),
                        CachedPipelineState::Ok(_),
                        CachedPipelineState::Ok(_),
                        CachedPipelineState::Ok(_),
                        CachedPipelineState::Ok(_),
                        CachedPipelineState::Ok(_),
                        CachedPipelineState::Ok(_),
                        CachedPipelineState::Ok(_),
                        CachedPipelineState::Ok(_),
                        CachedPipelineState::Ok(_),
                    ) => self.state = FluidState::Update,
                    _ => {}
                }
            }
            FluidState::Update => {}
        }
    }

    fn run<'w>(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), render_graph::NodeRunError> {
        let advection_pipeline = world.resource::<AdvectionPipeline>();
        let grid_label_pipeline = world.resource::<GridLabelPipeline>();
        let levelset_pipeline = world.resource::<AdvectLevelsetPipeline>();
        let recompute_levelset_pipeline = world.resource::<RecomputeLevelsetPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let uniform_bind_group = &world.resource::<SimulationUniformBindGroup>().0;
        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        match self.state {
            FluidState::Loading => {}
            FluidState::Init => {
                let grid_label_pipeline = pipeline_cache
                    .get_compute_pipeline(grid_label_pipeline.init_pipeline)
                    .unwrap();
                let grid_label_bind_group = &world.resource::<GridLabelBindGroup>().0;
                pass.set_pipeline(&grid_label_pipeline);
                pass.set_bind_group(0, grid_label_bind_group, &[]);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);

                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(advection_pipeline.init_pipeline)
                    .unwrap();
                let velocity_bind_group = &world.resource::<VelocityBindGroup>().0;
                let intermediate_velocity_bind_group =
                    &world.resource::<IntermediateVelocityBindGroup>().0;
                pass.set_pipeline(init_pipeline);
                pass.set_bind_group(0, velocity_bind_group, &[]);
                pass.set_bind_group(1, intermediate_velocity_bind_group, &[]);
                pass.set_bind_group(2, uniform_bind_group, &[]);
                pass.set_bind_group(3, grid_label_bind_group, &[]);
                pass.dispatch_workgroups(SIZE.0 + 1, SIZE.1 / WORKGROUP_SIZE / WORKGROUP_SIZE, 1);

                let init_levelset_pipeline = pipeline_cache
                    .get_compute_pipeline(levelset_pipeline.init_pipeline)
                    .unwrap();
                let levelset_bind_group = &world.resource::<LevelSetBindGroup>().0;
                pass.set_pipeline(init_levelset_pipeline);
                pass.set_bind_group(0, levelset_bind_group, &[]);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
            FluidState::Update => {
                let grid_label_pipeline = pipeline_cache
                    .get_compute_pipeline(grid_label_pipeline.update_pipeline)
                    .unwrap();
                let circle_collection_bind_group = &world.resource::<CircleCollectionBindGroup>().0;
                let grid_label_bind_group = &world.resource::<GridLabelBindGroup>().0;
                let levelset_bind_group = &world.resource::<LevelSetBindGroup>().0;

                pass.set_pipeline(grid_label_pipeline);
                pass.set_bind_group(0, grid_label_bind_group, &[]);
                pass.set_bind_group(1, circle_collection_bind_group, &[]);
                pass.set_bind_group(2, levelset_bind_group, &[]);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);

                let advection_pipeline = pipeline_cache
                    .get_compute_pipeline(advection_pipeline.pipeline)
                    .unwrap();

                let velocity_bind_group = &world.resource::<VelocityBindGroup>().0;
                let intermediate_velocity_bind_group =
                    &world.resource::<IntermediateVelocityBindGroup>().0;
                pass.set_pipeline(advection_pipeline);
                pass.set_bind_group(0, velocity_bind_group, &[]);
                pass.set_bind_group(1, intermediate_velocity_bind_group, &[]);
                pass.set_bind_group(2, uniform_bind_group, &[]);
                pass.set_bind_group(3, grid_label_bind_group, &[]);
                pass.dispatch_workgroups(SIZE.0 + 1, SIZE.1 / WORKGROUP_SIZE / WORKGROUP_SIZE, 1);

                // Add force if triggered.
                let add_force_pipeline = world.resource::<AddForcePipeline>();
                let add_force_pipeline = pipeline_cache
                    .get_compute_pipeline(add_force_pipeline.pipeline)
                    .unwrap();
                let local_force_bind_group = &world.resource::<LocalForceBindGroup>().0;
                pass.set_pipeline(add_force_pipeline);
                pass.set_bind_group(0, local_force_bind_group, &vec![]);
                pass.set_bind_group(2, uniform_bind_group, &[]);
                pass.set_bind_group(3, levelset_bind_group, &[]);
                pass.dispatch_workgroups(SIZE.0 + 1, SIZE.1 / WORKGROUP_SIZE / WORKGROUP_SIZE, 1);

                let divergence_pipeline = world.resource::<DivergencePipeline>();
                let divergence_pipeline = pipeline_cache
                    .get_compute_pipeline(divergence_pipeline.pipeline)
                    .unwrap();
                let divergence_bind_group = &world.resource::<DivergenceBindGroup>().0;
                pass.set_bind_group(0, divergence_bind_group, &[]);
                pass.set_bind_group(2, grid_label_bind_group, &[]);
                pass.set_pipeline(&divergence_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);

                let jacobi_pipeline_bundle = world.resource::<JacobiPipeline>();
                let jacobi_pipeline = pipeline_cache
                    .get_compute_pipeline(jacobi_pipeline_bundle.pipeline)
                    .unwrap();
                let swap_pipeline = pipeline_cache
                    .get_compute_pipeline(jacobi_pipeline_bundle.swap_pipeline)
                    .unwrap();

                let pressure_bind_group = &world.resource::<PressureBindGroup>().0;
                let intermediate_pressure_bind_group = &world.resource::<PressureBindGroup>().0;

                pass.set_bind_group(0, pressure_bind_group, &[]);
                pass.set_bind_group(1, intermediate_pressure_bind_group, &[]);
                pass.set_bind_group(2, uniform_bind_group, &[]);
                pass.set_bind_group(3, grid_label_bind_group, &[]);
                pass.set_bind_group(4, divergence_bind_group, &[]);
                for _ in 0..10 {
                    pass.set_pipeline(&jacobi_pipeline);
                    pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
                    pass.set_pipeline(&swap_pipeline);
                    pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
                }

                let solve_pipeline = world.resource::<SolvePressurePipeline>();
                let solve_pipeline = pipeline_cache
                    .get_compute_pipeline(solve_pipeline.pipeline)
                    .unwrap();

                pass.set_pipeline(&solve_pipeline);
                pass.set_bind_group(0, intermediate_velocity_bind_group, &[]);
                pass.set_bind_group(1, velocity_bind_group, &[]);
                pass.set_bind_group(2, pressure_bind_group, &[]);
                pass.set_bind_group(3, uniform_bind_group, &[]);
                pass.set_bind_group(4, grid_label_bind_group, &[]);
                pass.dispatch_workgroups(SIZE.0 + 1, SIZE.1 / WORKGROUP_SIZE / WORKGROUP_SIZE, 1);

                pass.set_bind_group(2, uniform_bind_group, &[]);

                let initialize_seeds_pipeline = pipeline_cache
                    .get_compute_pipeline(recompute_levelset_pipeline.initialize_pipeline)
                    .unwrap();
                let seeds_bind_group = &world.resource::<JumpFoodingSeedsBindGroup>().0;
                pass.set_pipeline(&initialize_seeds_pipeline);
                pass.set_bind_group(0, &seeds_bind_group, &[]);
                pass.set_bind_group(1, &levelset_bind_group, &[]);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);

                let jump_flood_pipeline = pipeline_cache
                    .get_compute_pipeline(recompute_levelset_pipeline.jump_flood_pipeline)
                    .unwrap();
                let jump_flood_uniform_bind_groups =
                    &world.resource::<JumpFloodingUniformBindGroups>();
                pass.set_pipeline(&jump_flood_pipeline);
                for uniform_bind_group in &jump_flood_uniform_bind_groups.bind_groups {
                    pass.set_bind_group(1, &uniform_bind_group, &[]);
                    pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
                }

                let sdf_pipeline = pipeline_cache
                    .get_compute_pipeline(recompute_levelset_pipeline.sdf_pipeline)
                    .unwrap();
                pass.set_pipeline(&sdf_pipeline);
                pass.set_bind_group(1, &levelset_bind_group, &[]);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);

                let levelset_pipeline = pipeline_cache
                    .get_compute_pipeline(levelset_pipeline.pipeline)
                    .unwrap();
                pass.set_pipeline(&levelset_pipeline);
                pass.set_bind_group(0, velocity_bind_group, &[]);
                pass.set_bind_group(1, levelset_bind_group, &[]);
                pass.set_bind_group(2, uniform_bind_group, &[]);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
        }

        Ok(())
    }
}

#[derive(Bundle, Default)]
pub struct SimulationBundle {
    pub name: Name,
    pub material: MaterialMeshBundle<FluidMaterial>,
}

#[cfg(test)]
mod test {
    #[test]
    fn rb32float_to_bytes() {
        let rg = &[0.0f32, 0.0f32];
        let bytes = bytemuck::bytes_of::<[f32; 2]>(rg);
        assert_eq!(bytes, &[0; 8]);
    }
}
