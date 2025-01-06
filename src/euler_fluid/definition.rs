use bevy::{
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        extract_resource::ExtractResource,
        render_resource::{AsBindGroup, ShaderType, UniformBuffer},
    },
};

/// Setting for fluid simulation. By spawning fluid settings, components required to the simulation will be spawned and the simulation will start.
/// Simulation result can be found on [`VelocityTextures`].
/// # Arguments
/// * `size`: The size of 2D simulation domain in pixels. The size is recommended to be same between each dimension and to be multiple of 64 pixels.
/// * `dx`: The size of a pixel in unit of [m/pixel].
/// * `dt`: The temporal resolution of the simulation in unit of [sec].
/// * `rho`: The density of fluid in unit of [kg/m^3]. Currently, uniform density is supported only.
/// * `initial_fluid_level`: Initialize fluid level with specified value. the value is valid between 0.0 - 1.0. 0.0 indicates empty and 1.0 indicates the simulation domain is filled with fluid.
/// * `gravity`: Uniform force enforced uniformly to the simulation domain in unit of [m/s^2].
/// 
/// # Examples
/// ```rust
/// use bevy::{
///     prelude::*,
///     sprite::MaterialMesh2dBundle,
/// };
/// use bevy_eulerian_fluid::euler_fluid::{
///     fluid_material::VelocityMaterial,
///     definition::{FluidSettings, VelocityTextures},
/// };
/// 
/// // On Startup
/// fn setup_scene(mut commands: Commands) {
///     commands.spawn(FluidSettings {
///         dx: 1.0f32,
///         dt: 0.5f32,
///         rho: 1.293f32,
///         gravity: Vec2::ZERO,
///         size: (512, 512),
///         initial_fluid_level: 1.0f32,
///     });
/// }
///
/// // On Update
/// fn on_advection_initialized(
///     mut commands: Commands,
///     query: Query<&VelocityTextures, Added<VelocityTextures>>,
///     mut meshes: ResMut<Assets<Mesh>>,
///     mut materials: ResMut<Assets<VelocityMaterial>>,
/// ) {
///     // Spawn a mesh to visualize fluid simulation.
///     for velocity_texture in &query {
///         let mesh = meshes.add(Rectangle::default());
///         let material = materials.add(VelocityMaterial {
///             offset: 0.5,
///             scale: 0.1,
///             u: Some(velocity_texture.u0.clone()),
///             v: Some(velocity_texture.v0.clone()),
///         });
///         commands
///             .spawn(MaterialMesh2dBundle {
///                 mesh: mesh.into(),
///                 transform: Transform::default().with_scale(Vec3::splat(512.0)),
///                 material,
///                 ..default()
///             });
///     }
/// }
/// ```
#[derive(Component, Clone, ExtractComponent)]
pub struct FluidSettings {
    pub dx: f32,
    pub dt: f32,
    pub rho: f32,
    pub gravity: Vec2,
    pub size: (u32, u32),
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

/// Fluid velocity field.
/// To retreive simulation result, please use u0 and v0.
/// u1, v1 are intermediate velocities used for simulation.
/// * u0: x-ward velocity with size of (size.0 + 1, size.1).
/// * v0: y-ward velocity with size of (size.0, size.1 + 1).
/// * u1: intermediate x-ward velocity with size of (size.0 + 1, size.1).
/// * v1: intermediate y-ward velocity with size of (size.0, size.1 + 1).
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
