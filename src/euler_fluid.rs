pub mod advection;
pub mod fluid_material;
use std::{borrow::Cow, vec};

use bevy::{
    prelude::*,
    render::{
        extract_component::{ComponentUniforms, ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin},
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        graph::CameraDriverLabel,
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{binding_types::{texture_storage_2d, uniform_buffer}, *},
        renderer::RenderDevice,
        Render,
        RenderApp,
        RenderSet
    },
};

use crate::texture::ImageForCS;

use self::fluid_material::FluidMaterial;

const SIZE: (u32, u32) = (512, 512);
const SIZE_VELOCITY: (u32, u32) = (SIZE.0 + 1, SIZE.1 + 1);
const WORKGROUP_SIZE: u32 = 8;

pub struct FluidPlugin;

impl Plugin for FluidPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<VelocityTexture>::default())
            .add_plugins(ExtractComponentPlugin::<SimulationUniform>::default())
            .add_plugins(UniformComponentPlugin::<SimulationUniform>::default())
            .add_plugins(MaterialPlugin::<FluidMaterial>::default())
            .add_systems(Startup, setup)
            .add_systems(Update, update);

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(Render, prepare_bind_group.in_set(RenderSet::PrepareBindGroups));

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node(FluidLabel, FluidNode::default());
        render_graph.add_node_edge(FluidLabel, CameraDriverLabel);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<FluidPipeline>();
    }
}

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<FluidPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    velocity_texture: Res<VelocityTexture>,
    render_device: Res<RenderDevice>,
    uniform: Res<ComponentUniforms<SimulationUniform>>,
) {
    let view = gpu_images.get(&velocity_texture.texture).unwrap();
    let cache_texture = gpu_images.get(&velocity_texture.velocity_cache).unwrap();
    let pressure = gpu_images.get(&velocity_texture.pressure).unwrap();
    let intermediate_pressure = gpu_images.get(&velocity_texture.intermediate_pressure).unwrap();
    let boundary_condition = gpu_images.get(&velocity_texture.boundary_condition).unwrap();
    let uniform = uniform.uniforms().binding().unwrap();

    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.texture_bind_group_layout,
        &BindGroupEntries::sequential((
            &view.texture_view,
            &cache_texture.texture_view,
            &pressure.texture_view,
            &intermediate_pressure.texture_view,
            &boundary_condition.texture_view,
            uniform))
    );

    commands.insert_resource(VelocityBindGroup(bind_group));
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FluidMaterial>>,
) {
    let velocity_image = Image::new_texture_storage(SIZE_VELOCITY, TextureFormat::Rg32Float);
    let velocity_image = images.add(velocity_image);

    let velocity_cache_image = Image::new_texture_storage(SIZE_VELOCITY, TextureFormat::Rg32Float);
    let velocity_cache_image = images.add(velocity_cache_image);

    let pressure = Image::new_texture_storage(SIZE, TextureFormat::R32Float);
    let pressure = images.add(pressure);

    let intermediate_pressure = Image::new_texture_storage(SIZE, TextureFormat::R32Float);
    let intermediate_pressure = images.add(intermediate_pressure);

    let boundary_condition = Image::new_texture_storage(SIZE, TextureFormat::R32Float);
    let boundary_condition = images.add(boundary_condition);

    let mesh = meshes.add(
        Mesh::from(Plane3d::default())
    );
    
    let material = materials.add(FluidMaterial {
        base_color: Color::RED,
        velocity_texture: Some(velocity_image.clone()),
    });

    commands.spawn(SimulationBundle {
        name: Name::new("sim"),
        material: MaterialMeshBundle {
            mesh,
            material,
            ..default()
        }
    });

    commands.insert_resource(VelocityTexture { texture: velocity_image, velocity_cache: velocity_cache_image, pressure, intermediate_pressure, boundary_condition});
    commands.spawn(SimulationUniform { dx: 1.0f32, dt: 0.1f32, rho: 1.293f32 });
}

fn update(
    mut query: Query<&mut SimulationUniform>,
    _time: Res<Time>,
) {
    for mut uniform in &mut query {
        uniform.dt = 0.1;
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct FluidLabel;

#[derive(Resource, Clone, ExtractResource, AsBindGroup)]
struct VelocityTexture {
    #[storage_texture(0, image_format = Rg32Float, access = ReadWrite)]
    texture: Handle<Image>,
    #[storage_texture(1, image_format = Rg32Float, access = ReadWrite)]
    velocity_cache: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadWrite)]
    pressure: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = ReadWrite)]
    intermediate_pressure: Handle<Image>,
    #[storage_texture(4, image_format = R32Float, access = ReadWrite)]
    boundary_condition: Handle<Image>,
}

#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
struct SimulationUniform {
    dx: f32,
    dt: f32,
    rho: f32,
}

#[derive(Resource)]
struct VelocityBindGroup(BindGroup);

#[derive(Resource)]
struct FluidPipeline {
    texture_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
    solve_pressure_pipeline: CachedComputePipelineId,
    update_pressure_pipeline: CachedComputePipelineId,
    update_velocity_init_pipeline: CachedComputePipelineId,
    update_velocity_add_pipeline: CachedComputePipelineId,
    update_velocity_sub_pipeline: CachedComputePipelineId,
    update_velocity_sub_v_pipeline: CachedComputePipelineId,
}

impl FromWorld for FluidPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let texture_bind_group_layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    texture_storage_2d(TextureFormat::Rg32Float, StorageTextureAccess::ReadWrite),
                    texture_storage_2d(TextureFormat::Rg32Float, StorageTextureAccess::ReadWrite),
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::ReadWrite),
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::ReadWrite),
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::ReadWrite),
                    uniform_buffer::<SimulationUniform>(false),
                ),
            ),
        );
        let shader = world.resource::<AssetServer>().load("shaders/euler_fluid.wgsl");

        let pipeline_cache = world.resource::<PipelineCache>();
        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("init"),
        });

        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });

        let solve_pressure_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("solve_pressure"),
        });

        let update_pressure_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("update_pressure"),
        });

        let update_velocity_init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("solve_velocity_init"),
        });

        let update_velocity_sub_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("solve_velocity_sub"),
        });

        let update_velocity_sub_v_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("solve_velocity_sub_v"),
        });

        let update_velocity_add_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("solve_velocity_add"),
        });

        FluidPipeline {
            texture_bind_group_layout,
            init_pipeline,
            update_pipeline,
            solve_pressure_pipeline,
            update_pressure_pipeline,
            update_velocity_init_pipeline,
            update_velocity_add_pipeline,
            update_velocity_sub_pipeline,
            update_velocity_sub_v_pipeline,
        }
    }
}

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
        let pipeline = world.resource::<FluidPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        match self.state {
            FluidState::Loading => {
                if let CachedPipelineState::Ok(_) = pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline) {
                    self.state = FluidState::Init;
                }
            }
            FluidState::Init => {
                if let CachedPipelineState::Ok(_) = pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline) {
                    self.state = FluidState::Update
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
        let texture_bind_group = &world.resource::<VelocityBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<FluidPipeline>();
        
        let mut pass = render_context.command_encoder().begin_compute_pass(&ComputePassDescriptor::default());
        pass.set_bind_group(0, texture_bind_group, &[]);

        match self.state {
            FluidState::Loading => {}
            FluidState::Init => {
                let init_pipeline = pipeline_cache.get_compute_pipeline(pipeline.init_pipeline).unwrap();
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
            FluidState::Update => {
                let update_pipeline = pipeline_cache.get_compute_pipeline(pipeline.update_pipeline).unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);

                let pressure_pipeline = pipeline_cache.get_compute_pipeline(pipeline.solve_pressure_pipeline).unwrap();
                let update_pressure_pipeline = pipeline_cache.get_compute_pipeline(pipeline.update_pressure_pipeline).unwrap();
                for _ in 0..50 {
                    pass.set_pipeline(pressure_pipeline);
                    pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
                    pass.set_pipeline(update_pressure_pipeline);
                    pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
                }

                // pass.set_pipeline(update_pressure_pipeline);
                // pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);

                let update_velocity_pipeline = pipeline_cache.get_compute_pipeline(pipeline.update_velocity_init_pipeline).unwrap();
                pass.set_pipeline(update_velocity_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);

                let update_velocity_sub_pipeline = pipeline_cache.get_compute_pipeline(pipeline.update_velocity_sub_pipeline).unwrap();
                pass.set_pipeline(update_velocity_sub_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);

                let update_velocity_sub_v_pipeline = pipeline_cache.get_compute_pipeline(pipeline.update_velocity_sub_v_pipeline).unwrap();
                pass.set_pipeline(update_velocity_sub_v_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);

                let update_velocity_add_pipeline = pipeline_cache.get_compute_pipeline(pipeline.update_velocity_add_pipeline).unwrap();
                pass.set_pipeline(update_velocity_add_pipeline);
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
        let bytes = bytemuck::bytes_of::<[f32;2]>(rg);
        assert_eq!(bytes, &[0;8]);
    }
}