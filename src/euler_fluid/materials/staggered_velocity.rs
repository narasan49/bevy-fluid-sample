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

use super::prepare_bind_group::PrepareBindGroup;

#[derive(Resource, Clone, ExtractResource, AsBindGroup)]
pub struct StaggeredVelocityMaterial {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub u: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub v: Handle<Image>,
}

#[derive(Resource)]
pub struct VelocityBindGroup(pub BindGroup);

#[derive(Resource, Clone)]
pub struct VelocityBindGroupLayout(pub BindGroupLayout);

impl PrepareBindGroup<StaggeredVelocityMaterial> for VelocityBindGroupLayout {
    fn prepare_bind_group(
        mut commands: Commands,
        bind_group_layout: Res<VelocityBindGroupLayout>,
        gpu_images: Res<RenderAssets<GpuImage>>,
        textures: Res<StaggeredVelocityMaterial>,
        render_device: Res<RenderDevice>,
        fallback_image: Res<FallbackImage>,
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

        commands.insert_resource(VelocityBindGroup(bind_group));
    }
}

impl FromWorld for VelocityBindGroupLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout =
            StaggeredIntermediateVelocityMaterial::bind_group_layout(render_device);
        Self(bind_group_layout)
    }
}

#[derive(Resource, Clone, ExtractResource, AsBindGroup)]
pub struct StaggeredIntermediateVelocityMaterial {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub u: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub v: Handle<Image>,
}

#[derive(Resource)]
pub struct IntermediateVelocityBindGroup(pub BindGroup);

#[derive(Resource, Clone)]
pub struct IntermediateVelocityBindGroupLayout(pub BindGroupLayout);

impl FromWorld for IntermediateVelocityBindGroupLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let bind_group_layout =
            StaggeredIntermediateVelocityMaterial::bind_group_layout(render_device);
        Self(bind_group_layout)
    }
}

impl PrepareBindGroup<StaggeredIntermediateVelocityMaterial>
    for IntermediateVelocityBindGroupLayout
{
    fn prepare_bind_group(
        mut commands: Commands,
        bind_group_layout: Res<Self>,
        gpu_images: Res<RenderAssets<GpuImage>>,
        textures: Res<StaggeredIntermediateVelocityMaterial>,
        render_device: Res<RenderDevice>,
        fallback_image: Res<FallbackImage>,
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

        commands.insert_resource(IntermediateVelocityBindGroup(bind_group));
    }
}
