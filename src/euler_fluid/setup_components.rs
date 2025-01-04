use bevy::{prelude::*, render::render_resource::TextureFormat};

use crate::{
    euler_fluid::definition::{
        FluidSimulationBundle, LocalForces, PressureTextures, SimulationUniform, VelocityTextures,
    },
    texture::NewTexture,
};

use super::definition::{
    DivergenceTextures, FluidSettings, JumpFloodingSeedsTextures, LevelsetTextures,
};

pub(crate) fn watch_fluid_component(
    mut commands: Commands,
    query: Query<(Entity, &FluidSettings), Added<FluidSettings>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, settings) in &query {
        let size = settings.size;
        let size_u = (size.0 + 1, size.1);
        let size_v = (size.0, size.1 + 1);

        let u0 = images.new_texture_storage(size_u, TextureFormat::R32Float);
        let u1 = images.new_texture_storage(size_u, TextureFormat::R32Float);

        let v0 = images.new_texture_storage(size_v, TextureFormat::R32Float);
        let v1 = images.new_texture_storage(size_v, TextureFormat::R32Float);

        let div = images.new_texture_storage(size, TextureFormat::R32Float);

        let p0 = images.new_texture_storage(size, TextureFormat::R32Float);
        let p1 = images.new_texture_storage(size, TextureFormat::R32Float);

        let grid_label = images.new_texture_storage(size, TextureFormat::R32Uint);

        let levelset = images.new_texture_storage(size, TextureFormat::R32Float);
        let jump_flooding_seeds_x = images.new_texture_storage(size, TextureFormat::R32Float);
        let jump_flooding_seeds_y = images.new_texture_storage(size, TextureFormat::R32Float);

        let velocity_textures = VelocityTextures { u0, v0, u1, v1 };

        let pressure_textures = PressureTextures { p0, p1 };

        let divergence_textures = DivergenceTextures { div };

        let levelset_textures = LevelsetTextures {
            levelset,
            grid_label,
        };

        let uniform = SimulationUniform {
            dx: settings.dx,
            dt: settings.dt,
            rho: settings.rho,
            gravity: settings.gravity,
        };

        let local_forces = LocalForces {
            force: vec![],
            position: vec![],
        };

        let jump_flooding_seeds_textures = JumpFloodingSeedsTextures {
            jump_flooding_seeds_x,
            jump_flooding_seeds_y,
        };

        commands
            .entity(entity)
            .insert(FluidSimulationBundle {
                velocity_textures,
                pressure_textures,
                divergence_textures,
                local_forces,
                levelset_textures,
                jump_flooding_seeds_textures,
            })
            .insert(uniform);
    }
}
