#![allow(unused_variables)]
#![allow(unused_imports)]

/*#![allow(unused_imports)]

use crate::camera;

use cgmath::SquareMatrix;
use cgmath::InnerSpace;
use cgmath::Rotation3;
use cgmath::Zero;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    ortho_view: [[f32; 4]; 4],
}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            ortho_view: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_ortho(&mut self, camera: &camera::Camera, projection: &camera::Projection) {

        self.ortho_view = (projection.calc_matrix() * camera.view).into();
    }

    pub fn get_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("uniform_bind_group_layout"),
        });

        uniform_bind_group_layout
    }

    pub fn get_buffers(&mut self, device: &wgpu::Device) -> (wgpu::Buffer, wgpu::BindGroup) {

        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[self.ortho_view]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }
        );

        let uniform_bind_group_layout = Uniforms::get_bind_group_layout(device);

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }
            ],
            label: Some("uniform_bind_group"),
        });

        (uniform_buffer, uniform_bind_group)
    }
}
*/

use wgpu::util::DeviceExt;
use cgmath::SquareMatrix;
use cgmath::InnerSpace;
use cgmath::Rotation3;
use cgmath::Zero;

use crate::camera;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    time: f32,
}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            proj: cgmath::Matrix4::identity().into(),
            view: cgmath::Matrix4::identity().into(),
            model: cgmath::Matrix4::identity().into(),
            time: 0.0,
        }
    }

    pub fn update_view_proj(&mut self, camera: &camera::Camera, projection: &camera::Projection) {
        self.proj = projection.calc_matrix().into();
        self.view =  camera.view.into();
    }

    pub fn update_time(&mut self, time: f32) {
        self.time += time;
    } 

    pub fn update_model(&mut self, model: cgmath::Matrix4<f32>) {
        self.model = model.into();
    }
}