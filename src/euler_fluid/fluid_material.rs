use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};

#[derive(Asset, Clone, AsBindGroup, TypePath, Debug)]
pub struct VelocityMaterial {
    #[uniform(0)]
    pub offset: f32,
    #[uniform(1)]
    pub scale: f32,
    #[texture(2)]
    #[sampler(3)]
    pub u: Option<Handle<Image>>,
    #[texture(4)]
    #[sampler(5)]
    pub v: Option<Handle<Image>>,
}

impl Material for VelocityMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/render_velocity.wgsl".into()
    }
}

impl Material2d for VelocityMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/render_velocity_2d.wgsl".into()
    }
}
