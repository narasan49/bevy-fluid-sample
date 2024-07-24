use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_asset::RenderAssets,
        render_resource::{AsBindGroup, BindGroup, BindGroupLayout, ShaderType},
        renderer::RenderDevice,
        texture::{FallbackImage, GpuImage},
    },
};

#[derive(Component)]
pub struct Circle {
    pub radius: f32,
}

#[derive(Component)]
pub struct Velocity {
    pub u: f32,
    pub v: f32,
}

#[derive(Clone, Copy, ShaderType)]
pub struct CrircleUniform {
    pub r: f32,
    pub position: Vec2,
    pub velocity: Vec2,
}

#[derive(Resource, AsBindGroup, ExtractResource, Clone)]
pub struct CircleCollectionMaterial {
    #[storage(0, read_only, visibility(compute))]
    pub circles: Vec<CrircleUniform>,
}

#[derive(Resource, Clone)]
pub struct CircleCollectionBindGroupLayout(pub BindGroupLayout);

#[derive(Resource)]
pub struct CircleCollectionBindGroup(pub BindGroup);

pub fn prepare_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    bind_group_layout: Res<CircleCollectionBindGroupLayout>,
    material: Res<CircleCollectionMaterial>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    fallback_image: Res<FallbackImage>,
) {
    let bind_group = material
        .as_bind_group(
            &bind_group_layout.0,
            &render_device,
            &gpu_images,
            &fallback_image,
        )
        .unwrap()
        .bind_group;

    commands.insert_resource(CircleCollectionBindGroup(bind_group));
}
