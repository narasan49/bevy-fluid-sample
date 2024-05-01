use std::borrow::Cow;

use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        graph::CameraDriverLabel,
        render_asset::{RenderAssetUsages, RenderAssets},
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::*,
        renderer::RenderDevice,
        Render,
        RenderApp,
        RenderSet
    },
};

const SIZE: (u32, u32) = (512, 512);
const WORKGROUP_SIZE: u32 = 8;

pub struct FluidPlugin;

impl Plugin for FluidPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<VelocityTexture>::default())
            .add_plugins(MaterialPlugin::<FluidMaterial>::default())
            .add_systems(Startup, setup);

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
) {
    let view = gpu_images.get(&velocity_texture.texture).unwrap();
    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.texture_bind_group_layout,
        &BindGroupEntries::single(&view.texture_view)
    );

    commands.insert_resource(VelocityBindGroup(bind_group));
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FluidMaterial>>,
) {
    let mut image = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        }, 
        TextureDimension::D2,
        bytemuck::bytes_of(&[0.0f32;2]),
        TextureFormat::Rg32Float,
        RenderAssetUsages::RENDER_WORLD
    );

    image.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image = images.add(image);

    let mesh = meshes.add(
        Mesh::from(Plane3d::default())
    );
    
    let material = materials.add(FluidMaterial {
        base_color: Color::RED,
        velocity_texture: Some(image.clone()),
    });

    commands.spawn(SimulationBundle {
        name: Name::new("sim"),
        material: MaterialMeshBundle {
            mesh,
            material,
            ..default()
        }
    });

    commands.insert_resource(VelocityTexture { texture: image });
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct FluidLabel;

#[derive(Resource, Clone, Deref, ExtractResource, AsBindGroup)]
struct VelocityTexture {
    #[storage_texture(0, image_format = Rg32Float, access = ReadWrite)]
    texture: Handle<Image>,
}

#[derive(Resource)]
struct VelocityBindGroup(BindGroup);

#[derive(Resource)]
struct FluidPipeline {
    texture_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for FluidPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let texture_bind_group_layout = VelocityTexture::bind_group_layout(render_device);
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
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });

        FluidPipeline {
            texture_bind_group_layout,
            init_pipeline,
            update_pipeline,
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
            }
        }

        Ok(())
    }
}

#[derive(Asset, Clone, AsBindGroup, TypePath, Debug)]
pub struct FluidMaterial {
    #[uniform(0)]
    pub base_color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub velocity_texture: Option<Handle<Image>>,
}

impl Material for FluidMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/fluid_material.wgsl".into()
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