use bevy::{
    math::Vec2, prelude::{Component, Resource}, render::{
        extract_component::ExtractComponent,
        render_resource::{BindGroup, ShaderType},
    }
};

#[derive(Component, ExtractComponent, ShaderType, Clone, Copy, Default)]
pub struct DeltaTimeUniform {
    pub dt: f32,
}

#[derive(Resource)]
pub struct DeltaTimeUniformBindGroup(pub BindGroup);

#[derive(Component, ExtractComponent, ShaderType, Clone, Copy, Default)]
pub struct SimulationUniform {
    pub dx: f32,
    pub dt: f32,
    pub rho: f32,
    pub gravity: Vec2,
}

#[derive(Resource)]
pub struct SimulationUniformBindGroup(pub BindGroup);
