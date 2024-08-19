use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_resource::{AsBindGroup, BindGroup, BindGroupLayout}, renderer::RenderDevice,
    },
};

use super::prepare_bind_group::PrepareBindGroup;

#[derive(Resource, Clone, ExtractResource, AsBindGroup)]
pub struct LocalForceMaterial {
    #[storage(0, read_only, visibility(compute))]
    pub force: Vec<Vec2>,
    #[storage(1, read_only, visibility(compute))]
    pub position: Vec<Vec2>,
}

#[derive(Resource)]
pub struct LocalForceBindGroup(pub BindGroup);

#[derive(Resource)]
pub struct LocalForceBindGroupLayout(pub BindGroupLayout);

impl PrepareBindGroup<LocalForceMaterial> for LocalForceBindGroupLayout {
    fn prepare_bind_group(
        mut commands: Commands,
        bind_group_layout: Res<Self>,
        gpu_images: Res<bevy::render::render_asset::RenderAssets<bevy::render::texture::GpuImage>>,
        textures: Res<LocalForceMaterial>,
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

        commands.insert_resource(LocalForceBindGroup(bind_group));
    }
}

impl FromWorld for LocalForceBindGroupLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout =
            LocalForceMaterial::bind_group_layout(render_device);
        Self(bind_group_layout)
    }
}