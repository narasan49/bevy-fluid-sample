use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
};

const RENDER_VELOCITY_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x473523D890EA4717AC81C543D7D98CB6);

const RENDER_VELOCITY_2D_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0xC979D52C691249DE87FC93D4820BD57B);

pub struct FluidMaterialPlugin;

impl Plugin for FluidMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<VelocityMaterial>::default())
            .add_plugins(Material2dPlugin::<VelocityMaterial>::default());

        load_internal_asset!(
            app,
            RENDER_VELOCITY_SHADER_HANDLE,
            "material/shaders/render_velocity.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            RENDER_VELOCITY_2D_SHADER_HANDLE,
            "material/shaders/render_velocity_2d.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(Asset, Clone, AsBindGroup, TypePath, Debug)]
pub struct VelocityMaterial {
    #[uniform(0)]
    pub offset: f32,
    #[uniform(1)]
    pub scale: f32,
    #[texture(2)]
    #[sampler(3)]
    pub u: Handle<Image>,
    #[texture(4)]
    #[sampler(5)]
    pub v: Handle<Image>,
}

impl Material for VelocityMaterial {
    fn fragment_shader() -> ShaderRef {
        RENDER_VELOCITY_SHADER_HANDLE.into()
    }
}

impl Material2d for VelocityMaterial {
    fn fragment_shader() -> ShaderRef {
        RENDER_VELOCITY_2D_SHADER_HANDLE.into()
    }
}
