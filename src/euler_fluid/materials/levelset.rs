use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_resource::{AsBindGroup, BindGroup, BindGroupLayout},
        renderer::RenderDevice,
    },
};

use super::prepare_bind_group::PrepareBindGroup;

#[derive(Resource, Clone, ExtractResource, AsBindGroup)]
pub struct LevelSetMaterial {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub levelset: Handle<Image>,
}

#[derive(Resource)]
pub struct LevelSetBindGroup(pub BindGroup);

#[derive(Resource)]
pub struct LevelSetBindGroupLayout(pub BindGroupLayout);

impl PrepareBindGroup<LevelSetMaterial> for LevelSetBindGroupLayout {
    fn prepare_bind_group(
        mut commands: Commands,
        bind_group_layout: Res<Self>,
        gpu_images: Res<bevy::render::render_asset::RenderAssets<bevy::render::texture::GpuImage>>,
        textures: Res<LevelSetMaterial>,
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

        commands.insert_resource(LevelSetBindGroup(bind_group));
    }
}

impl FromWorld for LevelSetBindGroupLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = LevelSetMaterial::bind_group_layout(render_device);
        Self(bind_group_layout)
    }
}
