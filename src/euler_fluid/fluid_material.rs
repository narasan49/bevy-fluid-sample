use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

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
