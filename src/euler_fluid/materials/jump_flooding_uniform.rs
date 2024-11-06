use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::{
            binding_types::uniform_buffer, BindGroup, BindGroupEntries, BindGroupLayout,
            BindGroupLayoutEntries, ShaderStages, ShaderType, UniformBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
        Render, RenderApp, RenderSet,
    },
};

use crate::euler_fluid::SIZE;

pub struct JumpFloodPlugin;

impl Plugin for JumpFloodPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<JumpFloodingUniform>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<JumpFloodingUniformBuffer>()
            .add_systems(
                Render,
                prepare_jump_flood_resource.in_set(RenderSet::PrepareResources),
            )
            .add_systems(
                Render,
                JumpFloodingUniformBindGroupLayout::prepare_bind_group
                    .in_set(RenderSet::PrepareBindGroups),
            );
    }
}

#[derive(Resource, Default)]
pub struct JumpFloodingUniformBuffer {
    pub buffers: Vec<UniformBuffer<JumpFloodingUniform>>,
}

#[derive(Resource, ExtractResource, ShaderType, Clone, Default)]
pub struct JumpFloodingUniform {
    pub step: u32,
}

#[derive(Resource)]
pub struct JumpFloodingUniformBindGroups {
    pub bind_groups: Box<[BindGroup]>,
}

pub fn prepare_jump_flood_resource(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut jump_flooding_buffer: ResMut<JumpFloodingUniformBuffer>,
) {
    let max_power = ((SIZE.0.max(SIZE.1) as f32).log2() - 1.0).floor() as usize;
    let mut step = 2_u32.pow((max_power + 1) as u32);
    jump_flooding_buffer.buffers.resize_with(max_power + 1, || {
        step /= 2;
        UniformBuffer::from(JumpFloodingUniform { step })
    });
    for buffer in jump_flooding_buffer.buffers.iter_mut() {
        buffer.write_buffer(&render_device, &render_queue);
    }
}

#[derive(Resource)]
pub struct JumpFloodingUniformBindGroupLayout(pub BindGroupLayout);

impl JumpFloodingUniformBindGroupLayout {
    pub fn prepare_bind_group(
        mut commands: Commands,
        render_device: Res<bevy::render::renderer::RenderDevice>,
        uniforms: Res<JumpFloodingUniformBuffer>,
    ) {
        let bind_group_layout = render_device.create_bind_group_layout(
            Some("JumpFloodingUniformBindGroupLayout"),
            &BindGroupLayoutEntries::single(
                ShaderStages::COMPUTE,
                uniform_buffer::<JumpFloodingUniform>(false),
            ),
        );
        let mut bind_groups = Vec::with_capacity(uniforms.buffers.len());
        for buffer in &uniforms.buffers {
            bind_groups.push(render_device.create_bind_group(
                "JumpFloodingUniform bind group",
                &bind_group_layout,
                &BindGroupEntries::single(buffer.binding().unwrap()),
            ));
        }

        commands.insert_resource(JumpFloodingUniformBindGroups {
            bind_groups: bind_groups.into_boxed_slice(),
        });
    }
}
