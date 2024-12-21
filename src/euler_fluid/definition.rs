use bevy::prelude::*;

use super::advection::AdvectionTextures;

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

#[derive(Bundle)]
pub struct SimulationTextureBundle {
    pub advection_textures: AdvectionTextures,
}