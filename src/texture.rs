use bevy::render::{render_asset::RenderAssetUsages, render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages}, texture::Image};

pub trait ImageForCS {
    fn new_image(size: (u32, u32)) -> Self;
}

impl ImageForCS for Image {
    fn new_image(size: (u32, u32)) -> Self {
        let mut image = Image::new_fill(
            Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            }, 
            TextureDimension::D2,
            bytemuck::bytes_of(&[0.0f32;2]),
            TextureFormat::Rg32Float,
            RenderAssetUsages::RENDER_WORLD
        );
    
        image.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
        image
    }
}