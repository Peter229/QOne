#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

/*#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::r_backend;
use crate::texture;
use crate::r_render_pipeline;
use crate::camera;
use crate::uniform;

use wgpu::util::DeviceExt;

use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder},
};

pub struct State {
    surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    pub renderer: r_backend::Renderer,
    depth_texture: texture::Texture,
    pub camera: camera::Camera,
    projection: camera::Projection,
    uniforms: uniform::Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

impl State {

    pub async fn new(window: &Window) -> Self {

        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.unwrap();

        //Fifo or Immediate (vsync on and off)
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let depth_texture = texture::Texture::create_depth_texture(&device, &sc_desc, "depth_texture");

        let renderer = r_backend::Renderer::new(&device);

        let render_pipeline = r_render_pipeline::render_pipeline(&device, &sc_desc, wgpu::include_spirv!("shader.vert.spv"), wgpu::include_spirv!("shader.frag.spv"), r_backend::Vertex::desc());

        let camera = camera::Camera::new();

        let projection = camera::Projection::new(0.0, 16.0, 0.0, 9.0, 0.0, 10.0);

        let mut uniforms = uniform::Uniforms::new();
        uniforms.update_view_ortho(&camera, &projection);
    
        let (uniform_buffer, uniform_bind_group) = uniforms.get_buffers(&device);

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_pipeline,
            renderer,
            depth_texture,
            camera,
            projection,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {

        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.sc_desc, "depth_texture");
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {

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
                //self.camera.process_keyboard(*keycode, *state);
                true
            }
            WindowEvent::CursorMoved  { position, .. } => {
                true
            }
            WindowEvent::MouseInput { state, button, .. } => {
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self) {

        self.camera.update_camera();
        self.uniforms.update_view_ortho(&self.camera, &self.projection);
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniforms]));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {

        self.renderer.update_buffers(&self.device);

        let frame = self.swap_chain.get_current_frame()?.output;
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            for texture_vertices in self.renderer.render_info.values_mut() {
                render_pass.set_vertex_buffer(0, texture_vertices.buf.slice(..));
                render_pass.set_bind_group(1, &texture_vertices.bind_group, &[]);
                render_pass.draw(0..(texture_vertices.verts.len() as u32), 0..1);
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        self.renderer.clear_verts();
        Ok(())
    }
}*/

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
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    pub size: winit::dpi::PhysicalSize<u32>,
    qbsp_pipeline: wgpu::RenderPipeline,
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

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default, //HighPerformance
                compatible_surface: Some(&surface),
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                shader_validation: true,
            },
            None,
        ).await.unwrap();

        //Fifo or Immediate (vsync on and off) Mailbox (double buffering)
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let mut camera = camera::Camera::new();
        let projection = camera::Projection::new(sc_desc.width, sc_desc.height, cgmath::Deg(90.0), 0.1, 4000.0);
        let camera_controller = camera::CameraController::new(400.0, 3.0);

        let mut uniforms = uniform::Uniforms::new();
        uniforms.update_view_proj(&camera, &projection);

        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }
        );

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
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
                    resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
                }
            ],
            label: Some("uniform_bind_group"),
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Uint,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let lightmap_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2Array,
                        component_type: wgpu::TextureComponentType::Uint,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let mut qbsp = qone_bsp::Bsp::new(&device, &queue, &texture_bind_group_layout, &lightmap_bind_group_layout);

        let depth_texture = texture::Texture::create_depth_texture(&device, &sc_desc, "depth_texture");

        let qone_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let qbsp_pipeline = r_pipeline::pipeline(&device, &sc_desc, &qone_layout, wgpu::include_spirv!("qbsp.vert.spv"), wgpu::include_spirv!("qbsp.frag.spv"), qone_types::RVertex::desc(), wgpu::PrimitiveTopology::TriangleList);

        let mut qone_player = qone_player::Player::new();

        qone_player.position = cgmath::Vector3::new(466.8673, 234.07431, 99.91692);

        camera.position = cgmath::Point3::new(466.8673, 234.07431, 99.91692);

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            qbsp_pipeline,
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
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.projection.resize(new_size.width, new_size.height);
        self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.sc_desc, "depth_texture");
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
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
                self.camera_controller.process_mouse((position.x as f32 / self.sc_desc.width as f32) - 0.5, 0.5 - (position.y as f32 / self.sc_desc.height as f32), &mut self.camera);
                window.set_cursor_position(winit::dpi::PhysicalPosition::new(self.sc_desc.width as f32 / 2.0, self.sc_desc.height as f32 / 2.0));
            }
            WindowEvent::MouseInput { state, button, .. } => {

            }
            _ => {},
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
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniforms]));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError> {

        let frame = self.swap_chain.get_current_frame()?.output;
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut offset = 0usize;
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            
            let mut offset = 0;
            render_pass.set_pipeline(&self.qbsp_pipeline);
            render_pass.set_vertex_buffer(0, self.qbsp.vertex_buffer.slice(..));
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            for i in 0..self.qbsp.render_offsets.len() {
                if self.qbsp.render_offsets[i] != 0 {
                    render_pass.set_bind_group(1, &self.qbsp.textures[i].bind_group, &[]);
                    render_pass.draw(offset..(offset + self.qbsp.render_offsets[i]), 0..1);
                    offset += self.qbsp.render_offsets[i];
                }
            }

        }
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}