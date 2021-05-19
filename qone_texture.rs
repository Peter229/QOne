use crate::qone_types::*;
use crate::texture;

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
}

impl Colour {
    pub fn add_alpha(&mut self) -> ColourAlpha {
        ColourAlpha { r: self.r, g: self.g, b: self.b, a: 255 }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColourAlpha {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

pub fn load_pallete() -> Vec<Colour> {
    let mut dir = "./res/palette.lmp".to_string();
    let mut bytes = std::fs::read(dir).unwrap();
    let mut colours: Vec<Colour> = Vec::new();
    for i in (0..bytes.len()).step_by(3) {
        colours.push(*bytemuck::from_bytes::<Colour>(&bytes[i..(i + 3)]));
    }
    colours
}

pub fn parse_textures(entry: Entry, bytes: &[u8], palette: &Vec<Colour>, device: &wgpu::Device, queue: &wgpu::Queue, layout: &wgpu::BindGroupLayout) -> (Vec<Material>, Vec<MipTex>) {
    let mut textures: Vec<Material> = Vec::new();
    let mut mip_texs: Vec<MipTex> = Vec::new();
    let num_tex = *bytemuck::from_bytes::<i32>(&bytes[(entry.offset as usize)..(entry.offset as usize + std::mem::size_of::<i32>())]);
    let mut total_offset = 0;
    for i in 0..num_tex {
        let offset = *bytemuck::from_bytes::<i32>(&bytes[(entry.offset as usize + std::mem::size_of::<i32>() * (i as usize + 1))..(entry.offset as usize + std::mem::size_of::<i32>() * (i as usize + 2))]);
        let header_size = std::mem::size_of::<i32>() + (std::mem::size_of::<i32>() * (num_tex as usize));
        
        let mip_tex = *bytemuck::from_bytes::<MipTex>(&bytes[((entry.offset + offset) as usize)..((entry.offset + offset) as usize + std::mem::size_of::<MipTex>())]);
        
        let size = (mip_tex.width * mip_tex.height) as usize;
        let tex_name = std::str::from_utf8(&mip_tex.name).unwrap();
        let mut tex_type = TEX_DEBUG;
        if tex_name.find("clip").is_some() || tex_name.find("trigger").is_some() || size == 0 {
            tex_type = TEX_DEBUG;
        }
        else if tex_name.find("sky").is_some() {
            tex_type = TEX_SKY;
        }
        else if tex_name.find('*').is_some() {
            tex_type = TEX_FLUID;
        }
        else {
            tex_type = TEX_DEFAULT;
        }

        let start_data = ((entry.offset as usize) + mip_tex.offset_1 as usize) + offset as usize;
        let end_data = ((entry.offset as usize) + mip_tex.offset_1 as usize + size) + offset as usize;

        let texture_colour_indices = &bytes[(start_data)..(end_data)];

        let mut texels: Vec<ColourAlpha> = Vec::new();
        for ind in texture_colour_indices {
            texels.push(palette[*ind as usize].clone().add_alpha());
        }

        let diffuse_texture = texture::Texture::from_array_with_alpha(device, queue, bytemuck::cast_slice(&texels), mip_tex.width, mip_tex.height, "Level texture").unwrap();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: None,
        });
        //Save image due to renderdoc not working
        //let path = "./".to_string() + &i.to_string() + ".png";
        //image::save_buffer_with_format(path, bytemuck::cast_slice(&texels), mip_tex.width, mip_tex.height, image::ColorType::Rgba8, image::ImageFormat::Png).unwrap();
        mip_texs.push(mip_tex);
        textures.push(Material { diffuse_texture, bind_group, tex_type });
    }
    (textures, mip_texs)
}
