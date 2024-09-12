use std::borrow::Cow;

use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, BindGroup, BindGroupLayout, CachedComputePipelineId,
            ComputePipelineDescriptor, PipelineCache,
        },
        renderer::RenderDevice,
        texture::{FallbackImage, GpuImage},
    },
};

use super::{
    geometry::{CircleCollectionBindGroupLayout, CircleCollectionMaterial},
    materials::levelset::LevelSetBindGroupLayout,
};

#[derive(Resource, Clone, ExtractResource, AsBindGroup)]
pub struct GridLabelMaterial {
    // 0: Empty grid, 1: Fluid, 2: Solid
    #[storage_texture(0, image_format = R32Uint, access = ReadWrite)]
    pub grid_label: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub u_solid: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadWrite)]
    pub v_solid: Handle<Image>,
}

#[derive(Resource)]
pub struct GridLabelBindGroup(pub BindGroup);

#[derive(Resource, Clone)]
pub struct GridLabelBindGroupLayout(pub BindGroupLayout);

pub fn prepare_bind_group(
    mut commands: Commands,
    gpu_images: Res<RenderAssets<GpuImage>>,
    material: Res<GridLabelMaterial>,
    render_device: Res<RenderDevice>,
    fallback_image: Res<FallbackImage>,
) {
    let bind_group_layout = GridLabelMaterial::bind_group_layout(&render_device);
    let bind_group = material
        .as_bind_group(
            &bind_group_layout,
            &render_device,
            &gpu_images,
            &fallback_image,
        )
        .unwrap()
        .bind_group;

    commands.insert_resource(GridLabelBindGroup(bind_group));
}

#[derive(Resource)]
pub struct GridLabelPipeline {
    pub init_pipeline: CachedComputePipelineId,
    pub update_pipeline: CachedComputePipelineId,
}

impl FromWorld for GridLabelPipeline {
    fn from_world(world: &mut World) -> Self {
        let grid_label_bind_group_layout = world.get_resource::<GridLabelBindGroupLayout>();
        let grid_label_bind_group_layout = match grid_label_bind_group_layout {
            Some(value) => value.clone(),
            None => {
                let render_device = world.resource::<RenderDevice>();
                let layout = GridLabelBindGroupLayout {
                    0: GridLabelMaterial::bind_group_layout(render_device),
                };
                world.insert_resource(layout.clone());
                world.resource::<GridLabelBindGroupLayout>().clone()
            }
        };

        let circle_collection_bind_group_layout =
            world.get_resource::<CircleCollectionBindGroupLayout>();
        let circle_collection_bind_group_layout = match circle_collection_bind_group_layout {
            Some(value) => value.clone(),
            None => {
                let render_device = world.resource::<RenderDevice>();
                let layout = CircleCollectionBindGroupLayout {
                    0: CircleCollectionMaterial::bind_group_layout(render_device),
                };
                world.insert_resource(layout.clone());
                world.resource::<CircleCollectionBindGroupLayout>().clone()
            }
        };
        let levelset_bind_group_layout = &world.resource::<LevelSetBindGroupLayout>().0;

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/grid_label.wgsl");

        let pipeline_cache = world.resource::<PipelineCache>();
        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("GridLabel Pipeline")),
            layout: vec![grid_label_bind_group_layout.0.clone()],
            push_constant_ranges: vec![],
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("initialize"),
        });

        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("GridLabel UpdatePipeline")),
            layout: vec![
                grid_label_bind_group_layout.0.clone(),
                circle_collection_bind_group_layout.0.clone(),
                levelset_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });

        Self {
            init_pipeline,
            update_pipeline,
        }
    }
}
