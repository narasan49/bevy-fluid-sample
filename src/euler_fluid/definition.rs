use bevy::{
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        extract_resource::ExtractResource,
        render_resource::{AsBindGroup, ShaderType, UniformBuffer},
    },
};

#[derive(Component, Clone, ExtractComponent)]
pub struct FluidSettings {
    pub dx: f32,
    pub dt: f32,
    pub rho: f32,
    pub gravity: Vec2,
    pub size: (u32, u32),
    /// Initialize fluid level with specified value.
    /// Valid range: 0.0 (empty) - 1.0 (filled with fluid).
    pub initial_fluid_level: f32,
}

#[derive(Component, ExtractComponent, ShaderType, Clone, Copy, Default)]
pub struct SimulationUniform {
    pub dx: f32,
    pub dt: f32,
    pub rho: f32,
    pub gravity: Vec2,
    pub initial_fluid_level: f32,
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
pub struct PressureTextures {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub p0: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub p1: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct DivergenceTextures {
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub div: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct LevelsetTextures {
    // levelset between fluid and empty grids. 0: fluid interface, positive: empty grids, negative: fluid grids.
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub levelset: Handle<Image>,
    // grid label which describe grid state. 0: empty, 1: fluid, 2: solid.
    #[storage_texture(1, image_format = R32Uint, access = ReadWrite)]
    pub grid_label: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct LocalForces {
    #[storage(0, read_only, visibility(compute))]
    pub force: Vec<Vec2>,
    #[storage(1, read_only, visibility(compute))]
    pub position: Vec<Vec2>,
}

#[derive(Clone, ShaderType)]
pub struct CircleObstacle {
    pub radius: f32,
    pub center: Vec2,
    pub velocity: Vec2,
}

#[derive(Resource, Clone, ExtractResource, AsBindGroup)]
pub struct Obstacles {
    #[storage(0, read_only, visibility(compute))]
    pub circles: Vec<CircleObstacle>,
}

impl FromWorld for Obstacles {
    fn from_world(_world: &mut World) -> Self {
        Self { circles: vec![] }
    }
}

#[derive(Component, Clone, ExtractComponent, AsBindGroup)]
pub struct JumpFloodingSeedsTextures {
    /// Note: Only R32Float, R32Sint, and R32Uint storage textures can have ReadWrite access on WebGPU.
    /// https://webgpufundamentals.org/webgpu/lessons/webgpu-storage-textures.html
    #[storage_texture(0, image_format = R32Float, access = ReadWrite)]
    pub jump_flooding_seeds_x: Handle<Image>,
    #[storage_texture(1, image_format = R32Float, access = ReadWrite)]
    pub jump_flooding_seeds_y: Handle<Image>,
}

#[derive(Component, Clone, ExtractComponent, ShaderType)]
pub struct JumpFloodingUniform {
    pub step: u32,
}

#[derive(Component)]
pub struct JumpFloodingUniformBuffer {
    pub buffer: Vec<UniformBuffer<JumpFloodingUniform>>,
}

#[derive(Bundle)]
pub struct FluidSimulationBundle {
    pub velocity_textures: VelocityTextures,
    pub pressure_textures: PressureTextures,
    pub divergence_textures: DivergenceTextures,
    pub levelset_textures: LevelsetTextures,
    pub local_forces: LocalForces,
    pub jump_flooding_seeds_textures: JumpFloodingSeedsTextures,
}
