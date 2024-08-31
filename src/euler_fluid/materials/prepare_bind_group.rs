use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::AsBindGroup,
        renderer::RenderDevice,
        texture::{FallbackImage, GpuImage},
    },
};
pub trait PrepareBindGroup<U: AsBindGroup + Resource>: Resource {
    fn prepare_bind_group(
        commands: Commands,
        bind_group_layout: Res<Self>,
        gpu_images: Res<RenderAssets<GpuImage>>,
        textures: Res<U>,
        render_device: Res<RenderDevice>,
        fallback_image: Res<FallbackImage>,
    );
}
