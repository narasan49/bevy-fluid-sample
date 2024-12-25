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
pub struct VelocityTextures {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub u0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub v0: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadWrite)]
    pub u1: Handle<Image>,
    #[storage_texture(3, image_format = R32Float, access = ReadWrite)]
    pub v1: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct GridCenterTextures {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub p0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub p1: Handle<Image>,
    #[storage_texture(2, image_format = R32Float, access = ReadWrite)]
    pub div: Handle<Image>,
    #[storage_texture(3, image_format = R32Uint, access = ReadWrite)]
    pub grid_label: Handle<Image>,
    #[storage_texture(4, image_format = R32Float, access = ReadWrite)]
    pub u_solid: Handle<Image>,
    #[storage_texture(5, image_format = R32Float, access = ReadWrite)]
    pub v_solid: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct LocalForces {
    #[storage(0, read_only, visibility(compute))]
    pub force: Vec<Vec2>,
    #[storage(1, read_only, visibility(compute))]
    pub position: Vec<Vec2>,
}

#[derive(Bundle)]
pub struct FluidSimulationBundle {
    pub velocity_textures: VelocityTextures,
    pub grid_center_textures: GridCenterTextures,
    pub local_forces: LocalForces,
}
