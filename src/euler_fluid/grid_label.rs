use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_asset::RenderAssets,
        render_resource::{AsBindGroup, BindGroup, BindGroupLayout},
        renderer::RenderDevice,
        texture::{FallbackImage, GpuImage},
    },
};

#[derive(Resource, Clone, ExtractResource, AsBindGroup)]
pub struct GridLabelMaterial {
    // 0: Empty grid, 1: Fluid, 2: Solid
    #[storage_texture(0, image_format = R32Uint, access = ReadWrite)]
    pub grid_label: Handle<Image>,
}

#[derive(Resource)]
pub struct GridLabelBindGroup(pub BindGroup);

#[derive(Resource)]
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
