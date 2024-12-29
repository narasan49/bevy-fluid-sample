use bevy::{
    prelude::Component,
    render::{extract_component::ExtractComponent, render_resource::ShaderType},
};

#[derive(Component, ExtractComponent, ShaderType, Clone, Copy, Default)]
pub struct SimulationUniform {
    pub dx: f32,
    pub dt: f32,
    pub rho: f32,
}
