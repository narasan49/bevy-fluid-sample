use std::borrow::Cow;

use bevy::{
    prelude::*,
    render::{
        extract_component::{
            ComponentUniforms, ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin
        },
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        graph::CameraDriverLabel,
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{binding_types::uniform_buffer, AsBindGroup, BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId, CachedPipelineState, ComputePassDescriptor, ComputePipelineDescriptor, PipelineCache, ShaderStages, ShaderType, TextureFormat},
        renderer::RenderDevice,
        texture::FallbackImage,
        Render,
        RenderApp,
        RenderSet,
    }
};

use crate::texture::ImageForCS;

use super::fluid_material::FluidMaterial;

const SIZE: (u32, u32) = (512, 512);
const SIZE_VELOCITY: (u32, u32) = (SIZE.0 + 1, SIZE.1 + 1);
const WORKGROUP_SIZE: u32 = 8;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct AdvectionLabel;

pub struct AdvectionPlugin;

impl Plugin for AdvectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<AdvectionUniform>::default())
            .add_plugins(UniformComponentPlugin::<AdvectionUniform>::default())
            .add_plugins(ExtractResourcePlugin::<AdvectionRenderResource>::default())
            .add_systems(Startup, setup)
            .add_systems(Update, update);

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(Render, prepare_bind_group.in_set(RenderSet::PrepareBindGroups));

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node(AdvectionLabel, AdvectionNode::default());
        render_graph.add_node_edge(AdvectionLabel, CameraDriverLabel);
    }

    // finish build
    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<AdvectionPipeline>();
    }
}

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<AdvectionPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    textures: Res<AdvectionRenderResource>,
    uniform: Res<ComponentUniforms<AdvectionUniform>>,
    render_device: Res<RenderDevice>,
    fallback_image: Res<FallbackImage>
) {
    let bind_group = textures.as_bind_group(
        &pipeline.bind_group_layout,
        &render_device,
        &gpu_images,
        &fallback_image).unwrap().bind_group;

    let uniform = uniform.uniforms().binding().unwrap();
    let uniform_bind_group = render_device.create_bind_group(
        None,
        &pipeline.uniform_bind_group_layout,
        &BindGroupEntries::single(uniform),
    );

    commands.insert_resource(AdvectionTextureBindGroup(bind_group));
    commands.insert_resource(AdvectionUniformBindGroup(uniform_bind_group));
}

#[derive(Resource, Clone, ExtractResource, AsBindGroup)]
pub struct AdvectionRenderResource {
    #[storage_texture(0, image_format = Rg32Float, access = ReadWrite)]
    pub input_velocity: Handle<Image>,
    #[storage_texture(1, image_format = Rg32Float, access = ReadWrite)]
    output_velocity: Handle<Image>,
}

#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
struct AdvectionUniform {
    dt: f32,
}

#[derive(Resource)]
struct AdvectionTextureBindGroup(BindGroup);

#[derive(Resource)]
struct AdvectionUniformBindGroup(BindGroup);

#[derive(Resource)]
struct AdvectionPipeline {
    bind_group_layout: BindGroupLayout,
    uniform_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    pipeline: CachedComputePipelineId,
    swap_pipeline: CachedComputePipelineId,
}

impl FromWorld for AdvectionPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = AdvectionRenderResource::bind_group_layout(render_device);
        let uniform_bind_group_layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<AdvectionUniform>(false),
            ),
        );
        let shader = world.resource::<AssetServer>().load("shaders/advection.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![bind_group_layout.clone(), uniform_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("advection"),
        });

        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![bind_group_layout.clone()],
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
            uniform_bind_group_layout,
            init_pipeline,
            pipeline,
            swap_pipeline,
        }
    }
}

pub trait AdvectionMaterial {
    fn add_advection(&mut self) -> Handle<FluidMaterial>;
}

impl <'a>AdvectionMaterial for ResMut<'a, Assets<FluidMaterial>> {
    fn add_advection(&mut self) -> Handle<FluidMaterial> {
        self.add(FluidMaterial {
            base_color: Color::RED,
            velocity_texture: None,
        })
    }
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let input_velocity = Image::new_texture_storage(SIZE_VELOCITY, TextureFormat::Rg32Float);
    let input_velocity = images.add(input_velocity);

    let output_velocity = Image::new_texture_storage(SIZE_VELOCITY, TextureFormat::Rg32Float);
    let output_velocity = images.add(output_velocity);

    commands.insert_resource(AdvectionRenderResource { input_velocity, output_velocity});
    commands.spawn(AdvectionUniform{ dt: 0.1f32 });
}

fn update(
    mut query: Query<&mut AdvectionUniform>,
) {
    for mut uniform in &mut query {
        uniform.dt = 1.0;
    }
}

enum AdvectionState {
    Loading,
    Init,
    Update,
}

struct AdvectionNode {
    state: AdvectionState,
}

impl Default for AdvectionNode {
    fn default() -> Self {
        Self {
            state: AdvectionState::Loading,
        }
    }
}

impl render_graph::Node for AdvectionNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<AdvectionPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        match self.state {
            AdvectionState::Loading => {
                if let CachedPipelineState::Ok(_) = pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline) {
                    self.state = AdvectionState::Init;
                }
            }
            AdvectionState::Init => {
                if let CachedPipelineState::Ok(_) = pipeline_cache.get_compute_pipeline_state(pipeline.pipeline) {
                    self.state = AdvectionState::Update;
                }
            }
            AdvectionState::Update => {}
        }
    }

    fn run<'w>(
            &self,
            _graph: &mut render_graph::RenderGraphContext,
            render_context: &mut bevy::render::renderer::RenderContext<'w>,
            world: &'w World,
        ) -> Result<(), render_graph::NodeRunError> {
        let bind_group = &world.resource::<AdvectionTextureBindGroup>().0;
        let uniform_bind_group = &world.resource::<AdvectionUniformBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<AdvectionPipeline>();

        let mut pass = render_context.command_encoder().begin_compute_pass(&ComputePassDescriptor::default());
        pass.set_bind_group(0, bind_group, &[]);
        pass.set_bind_group(1, uniform_bind_group, &[]);

        match self.state {
            AdvectionState::Loading => {}
            AdvectionState::Init => {
                let init_pipeline = pipeline_cache.get_compute_pipeline(pipeline.init_pipeline).unwrap();
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
            AdvectionState::Update => {
                let update_pipeline = pipeline_cache.get_compute_pipeline(pipeline.pipeline).unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);

                let swap_pipeline = pipeline_cache.get_compute_pipeline(pipeline.swap_pipeline).unwrap();
                pass.set_pipeline(swap_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
                
            }
        }
        Ok(())
    }
}