#![allow(unused_variables)]
#![allow(unused_imports)]

use wgpu::util::DeviceExt;
use cgmath::SquareMatrix;
use cgmath::InnerSpace;
use cgmath::Rotation3;
use cgmath::Zero;

use crate::camera;

const light_style_consts: [&str; 12] = ["m", "mmnmmommommnonmmonqnmmo", "abcdefghijklmnopqrstuvwxyzyxwvutsrqponmlkjihgfedcba", "mmmmmaaaaammmmmaaaaaabcdefgabcdefg", "mamamamamama", "jklmnopqrstuvwxyzyxwvutsrqponmlkj", "nmonqnmomnmomomno", "mmmaaaabcdefgmmmmaaaammmaamm", "mmmaaammmaaammmabcdefaaaammmmabcdefmmmaaaa", "aaaaaaaazzzzzzzz", "mmamammmmammamamaaamammma", "abcdefghijklmnopqrrqponmlkjihgfedcba"];

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    light_style: [f32; 4*12],
    eye: [f32; 3],
    time: f32,
}

impl Uniforms {
    pub fn new() -> Self {
        let light_style = [0.0; 4*12];
        let eye = [0.0; 3];
        Self {
            proj: cgmath::Matrix4::identity().into(),
            view: cgmath::Matrix4::identity().into(),
            model: cgmath::Matrix4::identity().into(),
            eye,
            light_style,
            time: 0.0,
        }
    }

    pub fn update_view_proj(&mut self, camera: &camera::Camera, projection: &camera::Projection) {
        self.proj = projection.calc_matrix().into();
        self.view =  camera.view.into();
    }

    pub fn update_eye(&mut self, eye: [f32; 3]) {
        for i in 0..3 {
            self.eye[i] = eye[i];
        }
    }

    pub fn update_time(&mut self, time: f32) {
        self.time += time;
    } 

    pub fn update_model(&mut self, model: cgmath::Matrix4<f32>) {
        self.model = model.into();
    }
    
    pub fn update_lights(&mut self) {

        for i in 0..light_style_consts.len() {
            let style = light_style_consts[i];
            let frame: i32 = (self.time * 10.0) as i32 % style.len() as i32;
            self.light_style[i*4] = (((style.chars().nth(frame as usize).unwrap() as i32) - ('a' as i32)) as f32) / (('z' as i32 - 'a' as i32) as f32);
        }
    }
}