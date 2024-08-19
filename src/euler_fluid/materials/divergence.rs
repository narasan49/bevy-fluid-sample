use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_resource::{AsBindGroup, BindGroup, BindGroupLayout},
        renderer::RenderDevice,
    },
};

use super::prepare_bind_group::PrepareBindGroup;

#[derive(Resource, ExtractResource, AsBindGroup, Clone)]
pub struct DivergenceMaterial {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub div: Handle<Image>,
}

#[derive(Resource)]
pub struct DivergenceBindGroupLayout(pub BindGroupLayout);

#[derive(Resource)]
pub struct DivergenceBindGroup(pub BindGroup);

impl PrepareBindGroup<DivergenceMaterial> for DivergenceBindGroupLayout {
    fn prepare_bind_group(
        mut commands: Commands,
        bind_group_layout: Res<Self>,
        gpu_images: Res<bevy::render::render_asset::RenderAssets<bevy::render::texture::GpuImage>>,
        textures: Res<DivergenceMaterial>,
        render_device: Res<bevy::render::renderer::RenderDevice>,
        fallback_image: Res<bevy::render::texture::FallbackImage>,
    ) {
        let bind_group = textures
            .as_bind_group(
                &bind_group_layout.0,
                &render_device,
                &gpu_images,
                &fallback_image,
            )
            .unwrap()
            .bind_group;

        commands.insert_resource(DivergenceBindGroup(bind_group));
    }
}

impl FromWorld for DivergenceBindGroupLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = DivergenceMaterial::bind_group_layout(render_device);
        Self(bind_group_layout)
    }
}
