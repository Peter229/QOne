#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder},
};
use futures::executor::block_on;
use wgpu::util::DeviceExt;

use cgmath::SquareMatrix;

use crate::texture;
use crate::uniform;
use crate::camera;
use crate::r_pipeline;
use crate::qone_bsp;
use crate::qone_player;
use crate::qone_types;

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    qbsp_pipeline: wgpu::RenderPipeline,
    qbsp_sky_pipeline: wgpu::RenderPipeline,
    qbsp_fluid_pipeline: wgpu::RenderPipeline,
    qbsp_debug_pipeline: wgpu::RenderPipeline,
    camera: camera::Camera,
    projection: camera::Projection,
    camera_controller: camera::CameraController,
    uniforms: uniform::Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    depth_texture: texture::Texture,
    qbsp: qone_bsp::Bsp,
    qone_player: qone_player::Player,
}

impl State {

    pub async fn new(window: &Window) -> Self {

        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(), //HighPerformance
                compatible_surface: Some(&surface),
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
            },
            None,
        ).await.unwrap();

        let swapchain_format = surface.get_preferred_format(&adapter).unwrap();

        let mut config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        surface.configure(&device, &config);

        let mut camera = camera::Camera::new();
        let projection = camera::Projection::new(config.width, config.height, cgmath::Deg(90.0), 0.1, 10000.0);
        let camera_controller = camera::CameraController::new(400.0, 0.002);

        let mut uniforms = uniform::Uniforms::new();
        uniforms.update_view_proj(&camera, &projection);

        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
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

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: false,
                        },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false, filtering: false },
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let lightmap_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: false,
                        },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false, filtering: false },
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let mut qbsp = qone_bsp::Bsp::new(&device, &queue, &texture_bind_group_layout, &lightmap_bind_group_layout);

        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let qone_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout, &lightmap_bind_group_layout],
            push_constant_ranges: &[],
        });

        let qbsp_pipeline = r_pipeline::pipeline(&device, swapchain_format, &qone_layout, wgpu::include_spirv!("qbsp.vert.spv"), wgpu::include_spirv!("qbsp.frag.spv"), qone_types::RVertex::desc(), wgpu::PrimitiveTopology::TriangleList);

        let qbsp_sky_pipeline = r_pipeline::pipeline(&device, swapchain_format, &qone_layout, wgpu::include_spirv!("qbsp_sky.vert.spv"), wgpu::include_spirv!("qbsp_sky.frag.spv"), qone_types::RVertex::desc(), wgpu::PrimitiveTopology::TriangleList);

        let qbsp_fluid_pipeline = r_pipeline::pipeline(&device, swapchain_format, &qone_layout, wgpu::include_spirv!("qbsp_fluid.vert.spv"), wgpu::include_spirv!("qbsp_fluid.frag.spv"), qone_types::RVertex::desc(), wgpu::PrimitiveTopology::TriangleList);

        let qbsp_debug_pipeline = r_pipeline::pipeline(&device, swapchain_format, &qone_layout, wgpu::include_spirv!("qbsp_debug.vert.spv"), wgpu::include_spirv!("qbsp_debug.frag.spv"), qone_types::RVertex::desc(), wgpu::PrimitiveTopology::TriangleList);

        let mut qone_player = qone_player::Player::new();

        qone_player.position = cgmath::Vector3::new(466.8673, 234.07431, 99.91692);

        camera.position = cgmath::Point3::new(466.8673, 234.07431, 99.91692);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            qbsp_pipeline,
            qbsp_sky_pipeline,
            qbsp_fluid_pipeline,
            qbsp_debug_pipeline,
            camera,
            projection,
            camera_controller,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            depth_texture,
            qbsp,
            qone_player,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {

        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.projection.resize(new_size.width, new_size.height);
        self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        self.surface.configure(&self.device, &self.config);
    }

    pub fn input(&mut self, event: &WindowEvent, window: &Window) {

        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                if *state == ElementState::Pressed { 
                    if *keycode == VirtualKeyCode::F {
                        self.qone_player.noclip = true;
                    }
                    if *keycode == VirtualKeyCode::G {
                        self.qone_player.noclip = false;
                        self.qone_player.position = cgmath::Vector3::new(self.camera.position.x, self.camera.position.y, self.camera.position.z);
                    }
                }
                if self.qone_player.noclip {
                    self.camera_controller.process_keyboard(*keycode, *state);
                }
                else {
                    self.qone_player.process_keyboard(*keycode, *state);
                }
            }
            WindowEvent::CursorMoved  { position, .. } => {
                //self.camera_controller.process_mouse(((position.x as f32) / (self.config.width as f32)) - 0.5, 0.5 - ((position.y as f32) / (self.config.height as f32)), &mut self.camera);
                //window.set_cursor_position(winit::dpi::PhysicalPosition::new((self.config.width as f32) / 2.0, (self.config.height as f32) / 2.0)).unwrap();
            }
            WindowEvent::MouseInput { state, button, .. } => {

            }
            _ => {},
        }
    }

    pub fn raw_mouse(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion {
                delta
            } => {
                self.camera_controller.process_mouse(delta.0 as f32, -delta.1 as f32, &mut self.camera);
            }
            _ => (),
        }
    }

    pub fn update(&mut self, delta_time: f32) {

        self.camera_controller.update_camera(&mut self.camera, delta_time);
        if !self.qone_player.noclip {
            self.qone_player.delta_time = delta_time;
            self.qone_player.update(&mut self.camera);
            self.qone_player.player_move(&mut self.qbsp);
            self.camera.position = cgmath::Point3::new(self.qone_player.position[0], self.qone_player.position[1], self.qone_player.position[2] + 24.0);
            self.camera.update_view();
        }
        self.uniforms.update_view_proj(&self.camera, &self.projection);
        self.uniforms.update_time(delta_time);
        self.uniforms.update_lights();
        self.uniforms.update_eye(self.camera.position.into());
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniforms]));
    }

    pub fn render(&mut self) {

        let frame = match self.surface.get_current_frame() {
            Ok(frame) => frame,
            Err(_) => {
                self.surface.configure(&self.device, &self.config);
                self.surface
                    .get_current_frame()
                    .expect("Failed to acquire next surface texture!")
            }
        };

        let view = frame
            .output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut offset = 0usize;
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[
                    wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: true,
                        }
                    }
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            
            let mut offset = 0;
            let mut transparent: Vec<(usize, u32)> = Vec::new();
            render_pass.set_pipeline(&self.qbsp_pipeline);
            render_pass.set_vertex_buffer(0, self.qbsp.vertex_buffer.slice(..));
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_bind_group(2, &self.qbsp.light_material.bind_group, &[]);
            for i in 0..self.qbsp.render_offsets.len() {
                if self.qbsp.render_offsets[i] != 0 {
                    if self.qbsp.textures[i].transparent {
                        transparent.push((i, offset));
                        offset += self.qbsp.render_offsets[i];
                        continue;
                    }
                    if self.qbsp.textures[i].tex_type == qone_types::TEX_SKY {
                        render_pass.set_pipeline(&self.qbsp_sky_pipeline);
                    }
                    else if self.qbsp.textures[i].tex_type == qone_types::TEX_FLUID {
                        render_pass.set_pipeline(&self.qbsp_fluid_pipeline);
                    }
                    else if self.qbsp.textures[i].tex_type == qone_types::TEX_DEFAULT {
                        render_pass.set_pipeline(&self.qbsp_pipeline);
                    }
                    else {
                        render_pass.set_pipeline(&self.qbsp_debug_pipeline);
                    }
                    render_pass.set_bind_group(1, &self.qbsp.textures[i].bind_group, &[]);
                    render_pass.draw(offset..(offset + self.qbsp.render_offsets[i]), 0..1);
                    offset += self.qbsp.render_offsets[i];
                }
            }

            render_pass.set_pipeline(&self.qbsp_pipeline);
            for i in 0..transparent.len() {
                let (tex_index, offset_t) = transparent[i];
                render_pass.set_bind_group(1, &self.qbsp.textures[tex_index].bind_group, &[]);
                render_pass.draw(offset_t..(offset_t + self.qbsp.render_offsets[tex_index]), 0..1);
            }

        }
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}