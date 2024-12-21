use bevy::{
    prelude::*,
    render::{extract_component::ExtractComponent, render_resource::AsBindGroup},
};

pub enum SimulationInterval {
    Fixed(f32),
    Dynamic,
}

#[derive(Component)]
pub struct FluidSettings {
    pub dx: f32,
    pub dt: SimulationInterval,
    pub rho: f32,
    pub size: (u32, u32),
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct AdvectionTextures {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub u0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub u1: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadWrite)]
    pub v0: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = ReadWrite)]
    pub v1: Handle<Image>,
    #[storage_texture(4, image_format = R32Uint, access = ReadWrite)]
    pub grid_label: Handle<Image>,
    #[storage_texture(5, image_format = R32Float, access = ReadWrite)]
    pub u_solid: Handle<Image>,
    #[storage_texture(6, image_format = R32Float, access = ReadWrite)]
    pub v_solid: Handle<Image>,
}

#[derive(Bundle)]
pub struct SimulationTextureBundle {
    pub advection_textures: AdvectionTextures,
}
