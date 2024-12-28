use bevy::{prelude::*, render::render_resource::TextureFormat};

use crate::{
    euler_fluid::{
        definition::{FluidSimulationBundle, GridCenterTextures, LocalForces, VelocityTextures},
        uniform::SimulationUniform,
    },
    texture::NewTexture,
};

use super::definition::FluidSettings;

pub(crate) fn watch_fluid_compoent(
    mut commands: Commands,
    query: Query<(Entity, &FluidSettings), Added<FluidSettings>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, settings) in &query {
        info!("Fluid settings spawned!: {:?}", entity);
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

        let velocity_textures = VelocityTextures { u0, v0, u1, v1 };

        let grid_center_textures = GridCenterTextures {
            p0,
            p1,
            div,
            grid_label,
        };

        let uniform = SimulationUniform {
            dx: settings.dx,
            dt: settings.dt,
            rho: settings.rho,
        };

        let local_forces = LocalForces {
            force: vec![],
            position: vec![],
        };

        commands
            .entity(entity)
            .insert(FluidSimulationBundle {
                velocity_textures,
                grid_center_textures,
                local_forces,
            })
            .insert(uniform);
    }
}
