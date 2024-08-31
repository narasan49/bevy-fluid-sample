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
pub struct PressureMaterial {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub p: Handle<Image>,
}

#[derive(Resource)]
pub struct PressureBindGroup(pub BindGroup);

#[derive(Resource)]
pub struct PressureBindGroupLayout(pub BindGroupLayout);

impl PrepareBindGroup<PressureMaterial> for PressureBindGroupLayout {
    fn prepare_bind_group(
        mut commands: Commands,
        bind_group_layout: Res<Self>,
        gpu_images: Res<bevy::render::render_asset::RenderAssets<bevy::render::texture::GpuImage>>,
        textures: Res<PressureMaterial>,
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

        commands.insert_resource(PressureBindGroup(bind_group));
    }
}

impl FromWorld for PressureBindGroupLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = PressureMaterial::bind_group_layout(render_device);
        Self(bind_group_layout)
    }
}

#[derive(Resource, ExtractResource, AsBindGroup, Clone)]
pub struct IntermediatePressureMaterial {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub p: Handle<Image>,
}
#[derive(Resource)]
pub struct IntermediatePressureBindGroup(pub BindGroup);

#[derive(Resource)]
pub struct IntermediatePressureBindGroupLayout(pub BindGroupLayout);

impl PrepareBindGroup<IntermediatePressureMaterial> for IntermediatePressureBindGroupLayout {
    fn prepare_bind_group(
        mut commands: Commands,
        bind_group_layout: Res<Self>,
        gpu_images: Res<bevy::render::render_asset::RenderAssets<bevy::render::texture::GpuImage>>,
        textures: Res<IntermediatePressureMaterial>,
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

        commands.insert_resource(IntermediatePressureBindGroup(bind_group));
    }
}

impl FromWorld for IntermediatePressureBindGroupLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout = IntermediatePressureMaterial::bind_group_layout(render_device);
        Self(bind_group_layout)
    }
}
