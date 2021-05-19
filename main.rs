#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

//mod camera;
mod texture;
/*mod r_state;
mod uniform;
mod r_pipeline;
mod qone_bsp;
mod qone_player;
mod qone_texture;
mod qone_types;*/

use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder},
};

use futures::executor::block_on;
use wgpu::util::DeviceExt;
use cgmath::SquareMatrix;
use cgmath::InnerSpace;
use cgmath::Rotation3;
use cgmath::Zero;
use std::mem;
use std::time::{Instant, Duration};

use noise::*;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = block_on(instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default, //HighPerformance
            compatible_surface: Some(&surface),
        },
    )).unwrap();

    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            shader_validation: true,
        },
        None,
    )).unwrap();

    let mut data: Vec<u8> = Vec::new();
    let width = 128;
    let height = 128;
    let seed = 5;
    let noise1 = OpenSimplex::new().set_seed(seed ^ 679125833);
    let noise2 = OpenSimplex::new().set_seed(seed ^ 3274989671);
    let noise3 = OpenSimplex::new().set_seed(seed ^ 2776948319);
    let noise4 = OpenSimplex::new().set_seed(seed ^ 4054792873);
    let noise5 = OpenSimplex::new().set_seed(seed ^ 2100345001);
    for x in 0..width {
        for y in 0..height {
            let nx = x as f64 / (width as f64 * 0.1);
            let ny = y as f64 / (height as f64 * 0.1);
            let nz = 5.0 as f64;
            let e
                = 16.0 * (noise1.get([01.0 * nx, 01.0 * ny, 01.0 * nz]) + 1.0) / 2.0
                + 08.0 * (noise2.get([02.0 * nx, 02.0 * ny, 02.0 * nz]) + 1.0) / 2.0
                + 04.0 * (noise3.get([04.0 * nx, 04.0 * ny, 04.0 * nz]) + 1.0) / 2.0
                + 02.0 * (noise4.get([08.0 * nx, 08.0 * ny, 08.0 * nz]) + 1.0) / 2.0
                + 01.0 * (noise5.get([16.0 * nx, 16.0 * ny, 16.0 * nz]) + 1.0) / 2.0;
            let val = ((e / 31.0) * 256.0) as u8;
            //trace!("{},{} => {} -> {}", x, y, e, val);
            data.push(val);
            data.push(0);
            data.push(0);
            data.push(255);
        }
    }
    let diffuse_texture = texture::Texture::from_array_with_alpha(&device, &queue, bytemuck::cast_slice(&data), width, height, "test").unwrap();

    let path = "test.png";
    image::save_buffer_with_format(path, bytemuck::cast_slice(&data), width, height, image::ColorType::Rgba8, image::ImageFormat::Png).unwrap();

    /*env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("QONE");
    window.set_cursor_grab(true);
    window.set_cursor_visible(false);
    let mut state = block_on(r_state::State::new(&window));
    //let mut fps: i32 = 0;
    let mut run_time = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                state.input(event, &window);
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput {
                        input,
                        ..
                    } => {
                        match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        }
                    },
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                /*fps += 1;
                if run_time.elapsed().as_millis() >= 1000 {
                    println!("fps {}", fps);
                    fps = 0;
                    run_time = Instant::now();
                }*/

                let delta_time = (((run_time.elapsed().as_nanos() as f64 / 1000.0) / 1000.0) / 1000.0) as f32;
                //println!("FPS {}", 1.0 / delta_time);
                run_time = Instant::now();

                state.update(delta_time);
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            _ => {}
        }
    });*/
}
