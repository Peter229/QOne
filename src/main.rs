#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

mod camera;
mod texture;
mod r_state;
mod uniform;
mod r_pipeline;
mod qone_bsp;
mod qone_player;
mod qone_texture;
mod qone_types;

use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder, Fullscreen},
    monitor::{MonitorHandle, VideoMode},
};

  
use std::io::{stdin, stdout, Write};
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

    env_logger::init();
    let event_loop = EventLoop::new();
    let full = Fullscreen::Exclusive(prompt_for_video_mode(&prompt_for_monitor(&event_loop)));
    let window = WindowBuilder::new().with_fullscreen(Some(full)).build(&event_loop).unwrap();
    window.set_title("QONE");
    window.set_cursor_grab(true).unwrap();
    window.set_cursor_visible(false);
    let mut state = block_on(r_state::State::new(&window));
    //let mut fps: i32 = 0;
    let mut run_time = Instant::now();
    let mut render_time = 0.0;
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::DeviceEvent {
                ref event,
                ..
            } => {
                state.raw_mouse(event);
            }
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
                render_time += delta_time;
                // 1 / DESIRED FRAME RATE
                if render_time >= 0.00666666666 {
                    state.render();
                    render_time = 0.0;
                }
                /*match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }*/
            }
            _ => {}
        }
    });
}

fn prompt_for_monitor(event_loop: &EventLoop<()>) -> MonitorHandle {
    for (num, monitor) in event_loop.available_monitors().enumerate() {
        println!("Monitor #{}: {:?}", num, monitor.name());
    }

    print!("Please write the number of the monitor to use: ");
    stdout().flush().unwrap();

    let mut num = String::new();
    stdin().read_line(&mut num).unwrap();
    let num = num.trim().parse().ok().expect("Please enter a number");
    let monitor = event_loop
        .available_monitors()
        .nth(num)
        .expect("Please enter a valid ID");

    println!("Using {:?}", monitor.name());

    monitor
}

fn prompt_for_video_mode(monitor: &MonitorHandle) -> VideoMode {
    for (i, video_mode) in monitor.video_modes().enumerate() {
        println!("Video mode #{}: {}", i, video_mode);
    }

    print!("Please write the number of the video mode to use: ");
    stdout().flush().unwrap();

    let mut num = String::new();
    stdin().read_line(&mut num).unwrap();
    let num = num.trim().parse().ok().expect("Please enter a number");
    let video_mode = monitor
        .video_modes()
        .nth(num)
        .expect("Please enter a valid ID");

    println!("Using {}", video_mode);

    video_mode
}

/*
use std::io;
use std::io::prelude::*;
use std::fs::File;
use wgpu::util::DeviceExt;
use std::io::{stdin,stdout,Write};

use crate::qone_texture;
use crate::qone_types::*;
use crate::texture;

//https://www.gamers.org/dEngine/quake/spec/quake-spec34/qkspec_4.htm

pub const CONTENTS_EMPTY: i32 = -1;
pub const CONTENTS_SOLID: i32 = -2;
pub const CONTENTS_WATER: i32 = -3;
pub const CONTENTS_CLIP: i32 = -8;
const DIST_EPSILON: f32 = 0.03125;

pub struct Bsp {
    pub vertex_buffer: wgpu::Buffer,
    pub num_verts: u32,
    models: Vec<Model>,
    nodes: Vec<Node2>,
    leafs: Vec<Leaf2>,
    planes: Vec<Plane>,
    clip_nodes: Vec<Vec<ClipNode2>>,
    pub textures: Vec<Material>,
    pub render_offsets: Vec<u32>,
    pub entity_info: Vec<EntityInfo>,
    pub collidable: Vec<i32>,
    pub light_material: Material,
    hulls: Vec<Hull>,
}

//https://discord.com/channels/380484403458998276/382248269444808724/835817766228721664
impl Bsp {

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, layout: &wgpu::BindGroupLayout, light_layout: &wgpu::BindGroupLayout) -> Bsp {

        let map_name = "ad_swampy";

        let dir = "./res/".to_string() + map_name + ".bsp";

        let mut light_rgba = read_rgb_lights(map_name);

        let mut bytes = std::fs::read(dir).unwrap();

        //Load header
        let header = *bytemuck::from_bytes::<Header>(&bytes[..std::mem::size_of::<Header>()]);

        let mut nodes = Vec::new();
        let mut clip_nodes_loaded = Vec::new();
        let mut edges = Vec::new();
        let mut faces = Vec::new();
        let mut leafs = Vec::new();

        //Load BSP2 and BSP29 specific
        if bytes[0] == 'B' as u8 && bytes[1] == 'S' as u8 && bytes[2] == 'P' as u8 && bytes[3] == '2' as u8 {
            nodes = parse_data::<Node2>(header.nodes, &bytes);
            clip_nodes_loaded = parse_data::<ClipNode2>(header.clipnodes, &bytes);
            edges = parse_data::<Edge2>(header.edges, &bytes);
            faces = parse_data::<Face2>(header.faces, &bytes);
            leafs = parse_data::<Leaf2>(header.leaves, &bytes);
        }
        else {
            let nodes_bsp29 = parse_data::<Node>(header.nodes, &bytes);
            let clip_nodes_bsp29 = parse_data::<ClipNode>(header.clipnodes, &bytes);
            let edges_bsp29 = parse_data::<Edge>(header.edges, &bytes);
            let faces_bsp29 = parse_data::<Face>(header.faces, &bytes);
            let leafs_bsp29 = parse_data::<Leaf>(header.leaves, &bytes);
            let (nodes_t, clip_nodes_t, edges_t, faces_t, leaves_t) = to_bsp2(&nodes_bsp29, &clip_nodes_bsp29, &edges_bsp29, &faces_bsp29, &leafs_bsp29);
            nodes = nodes_t;
            clip_nodes_loaded = clip_nodes_t;
            edges = edges_t;
            faces = faces_t;
            leafs = leaves_t;
        }

        
        //Load data
        let vertices = parse_data::<Vertex>(header.vertices, &bytes);
        let models = parse_data::<Model>(header.models, &bytes);
        let ledges = parse_data::<i32>(header.ledges, &bytes);
        let mut light_maps: Vec<u8> = Vec::new();
        if light_rgba.len() == 0 {
            light_maps = parse_data::<u8>(header.lightmaps, &bytes);
        }
        let planes = parse_data::<Plane>(header.planes, &bytes);
        let surfaces = parse_data::<Surface>(header.texinfo, &bytes);
        let entities = parse_data::<u8>(header.entities, &bytes);
        let pallete = qone_texture::load_pallete();
        let (textures, mip_texs) = qone_texture::parse_textures(header.miptex, &bytes, &pallete, device, queue, layout);

        let diffuse_texture = if light_rgba.len() == 0 {
            texture::Texture::from_array_light_map(device, queue, &light_maps, (light_maps.len() as f32).sqrt().ceil() as u32, "lightmap").unwrap()
        }
        else {
            let dimension = ((light_rgba.len() / 4) as f32).sqrt().ceil() as u32;
            texture::Texture::from_array_rgb_light_map(device, queue, &mut light_rgba, dimension, "lightmap").unwrap()
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: light_layout,
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

        let light_material = Material { diffuse_texture, bind_group, tex_type: -1 };
    

        //let mut entity_info = Vec::new();
        //let mut collidable = vec![0; 1];

        let mut ent_string = String::new();

        for i in 0..entities.len() {
            ent_string.push(entities[i] as char);
        }

        let (entity_info, collidable) = extract_entities(&ent_string);

        //Build mesh
        let (render_verts, render_offsets) = build_mesh(&models, &vertices, &faces, &edges, &ledges, &surfaces, &textures, &mip_texs);

        //Generate buffers
        let vertex_buffer = generate_buffer(device, &render_verts);


        //Generate clipping info
        let mut hulls = Vec::new();
        let mut clip_nodes = Vec::new();

        let mut clip_nodes_temp = Vec::new();

        for i in 0..nodes.len() {
            let child_one = if nodes[i].children[0] < 0 {
                nodes[i].children[0]
            }
            else {
                nodes[i].children[0] - nodes.len() as i32
            };
            let child_two = if nodes[i].children[1] < 0 {
                nodes[i].children[1]
            }
            else {
                nodes[i].children[1] - nodes.len() as i32
            };
            clip_nodes_temp.push(ClipNode2 { plane_num: nodes[i].plane_num - planes.len() as i32, children: [child_one, child_two] });
        }

        clip_nodes.push(clip_nodes_temp);
        clip_nodes.push(clip_nodes_loaded);

        hulls.push(get_hull0(clip_nodes[0].len()));

        get_hulls(&mut hulls, 1, 0, clip_nodes[1].len());

        let num_verts = render_verts.len() as u32;
        Bsp { vertex_buffer, num_verts, models, nodes, leafs, planes, clip_nodes, textures, render_offsets, entity_info, collidable, light_material, hulls } 
    }

    //https://github.com/id-Software/Quake/blob/bf4ac424ce754894ac8f1dae6a3981954bc9852d/QW/client/pmove.c
    pub fn player_trace(&mut self, start_pos: cgmath::Vector3<f32>, end_pos: cgmath::Vector3<f32>) -> Trace {
        
        let mut total_trace = Trace::new(end_pos);

        for i in 0..self.collidable.len() {

            let offset = to_vec3(self.models[self.collidable[i] as usize].origin);
            let start_l = start_pos - offset;
            let end_l = end_pos - offset;

            let mut trace = Trace::new(end_pos);
            trace.all_solid = true;
            self.recusive_hull_check(i as i32, self.models[self.collidable[i] as usize].node_id1 as i32, 0.0, 1.0, start_l, end_l, &mut trace);

            if trace.all_solid {
                trace.starts_solid = true;
            }
            if trace.starts_solid {
			    trace.fraction = 0.0;
            }

            if trace.fraction < total_trace.fraction {
                trace.end_pos = trace.end_pos + offset;
                total_trace = trace;
                total_trace.ent = i as i32;
            }
        }

        if total_trace.ent >= 0 {
            //println!("{}", self.entity_info[total_trace.ent as usize]);
            //println!("{:?}", total_trace.end_pos);
        }

        total_trace
    }

    //https://github.com/id-Software/Quake/blob/bf4ac424ce754894ac8f1dae6a3981954bc9852d/QW/client/pmovetst.c
    pub fn recusive_hull_check(&mut self, id: i32, num: i32, p1f: f32, p2f: f32, p1: cgmath::Vector3<f32>, p2: cgmath::Vector3<f32>, trace: &mut Trace) -> bool {

        if num < 0 {

            if num != CONTENTS_SOLID {

                trace.all_solid = false;
                if num == CONTENTS_EMPTY {
                    trace.in_open = true;
                }
                else {
                    trace.in_water = true;
                }
            }
            else {
                trace.starts_solid = true;
            }
            return true;
        }

        if num < self.models[id as usize].node_id1 as i32 || num > self.models[id as usize].node_id2 as i32 {
            panic!("RECUSIVE HULL CHECK BAD NODE NUMBER");
        }

        let node = self.clip_nodes[num as usize];
        let plane = self.planes[node.plane_num as usize];

        let mut t1 = 0.0;
        let mut t2 = 0.0;

        if plane.ptype < 3 {
            t1 = p1[plane.ptype as usize] - plane.dist;
            t2 = p2[plane.ptype as usize] - plane.dist;
        }
        else {
            t1 = cgmath::dot(to_vec3(plane.normal), p1) - plane.dist;
            t2 = cgmath::dot(to_vec3(plane.normal), p2) - plane.dist;
        }

        if t1 >= 0.0 && t2 >= 0.0 {
            return self.recusive_hull_check(id, node.children[0], p1f, p2f, p1, p2, trace);
        }
        if t1 < 0.0 && t2 < 0.0 {
            return self.recusive_hull_check(id, node.children[1], p1f, p2f, p1, p2, trace);
        }

        let mut frac = 0.0;
        if t1 < 0.0 {
            frac = (t1 + DIST_EPSILON) / ( t1 - t2);
        }
        else {
            frac = (t1 - DIST_EPSILON) / ( t1 - t2);
        }

        if frac < 0.0 {
            frac = 0.0;
        }
        if frac > 1.0 {
            frac = 1.0;
        }

        let mut midf = p1f + (p2f - p1f) * frac;
        let mut mid = cgmath::Vector3::new(0.0, 0.0, 0.0);
        for i in 0..3 {
            mid[i] = p1[i] + frac * (p2[i]  - p1[i]);
        }

        let side = (t1 < 0.0);
        let mut side_n = node.children[0] as i32;
        if side {
            side_n = node.children[1] as i32;
        }

        if (!self.recusive_hull_check(id, side_n, p1f, midf, p1, mid, trace)) {
            return false;
        }

        let side_t = (t1 < 0.0);
        let mut side_tn = node.children[1] as i32;
        if side_t {
            side_tn = node.children[0] as i32;
        } 

        if self.hull_point_contents(id, side_tn, mid) != CONTENTS_SOLID {
            return self.recusive_hull_check(id, side_tn, midf, p2f, mid, p2, trace);
        }

        if trace.all_solid {
            return false;
        }

        if !side {
            trace.plane = plane;
        }
        else {
            trace.plane.normal = [-plane.normal[0], -plane.normal[1], -plane.normal[2]];
            trace.plane.dist = -plane.dist;
        }

        while self.hull_point_contents(id, self.models[id as usize].node_id1, mid) == CONTENTS_SOLID {
            frac -= 0.1;
            if frac < 0.0 {
                trace.fraction = midf;
                trace.end_pos = mid;
                return false;
            }
            midf = p1f + (p2f - p1f) * frac;
            for i in 0..3 {
                mid[i] = p1[i] + frac * (p2[i] - p1[i]);
            }
        }

        trace.fraction = midf;
        trace.end_pos = mid;

        false
    }

    pub fn point_contents(&mut self, p: cgmath::Vector3<f32>) -> i32 {

        let mut num: i32 = 0;

        while num >= 0 {
            
            if num < 0 || num > self.clip_nodes.len() as i32 - 1 {
                panic!("HULL POINT CONTENTS BAD NODE NUMBER");
            }

            let node = self.clip_nodes[num as usize];
            let plane = self.planes[node.plane_num as usize];

            let mut d = 0.0;

            if plane.ptype < 3 {
                d = p[plane.ptype as usize] - plane.dist;
            }
            else {
                d = cgmath::dot(to_vec3(plane.normal), p) - plane.dist;
            }

            if d < 0.0 {
                num = node.children[1] as i32;
            }
            else {
                num = node.children[0] as i32;
            }
        }

        num
    }

    pub fn hull_point_contents(&mut self, id: i32, mut num: i32, p: cgmath::Vector3<f32>) -> i32 {

        while num >= 0 {
            if num < 0 || num > self.clip_nodes.len() as i32 - 1 {
                panic!("HULL POINT CONTENTS BAD NODE NUMBER");
            }

            let node = self.clip_nodes[num as usize];
            let plane = self.planes[node.plane_num as usize];

            let mut d = 0.0;

            if plane.ptype < 3 {
                d = p[plane.ptype as usize] - plane.dist;
            }
            else {
                d = cgmath::dot(to_vec3(plane.normal), p) - plane.dist;
            }

            if d < 0.0 {
                num = node.children[1] as i32;
            }
            else {
                num = node.children[0] as i32;
            }
        }

        num
    }

    pub fn test_player_position(&mut self, pos: cgmath::Vector3<f32>) -> bool {

        for i in 0..self.models.len() {

            let offset = to_vec3(self.models[i].origin);
            let test = pos - offset;

            if self.hull_point_contents(i as i32, self.models[i].node_id1 as i32, test) == CONTENTS_SOLID {
                return false
            }
        }

        return true;
    }
}

//https://discord.com/channels/380484403458998276/382248269444808724/835630122241359924
fn parse_data<T>(entry: Entry, bytes: &[u8]) -> Vec<T> where T:
bytemuck::Pod
{
    bytes[(entry.offset as usize)..((entry.offset + entry.size) as usize)].chunks_exact(std::mem::size_of::<T>()).map(|chunk| *bytemuck::from_bytes::<T>(chunk)).collect()
}

fn generate_buffer(device: &wgpu::Device, vertices: &Vec<RVertex>) -> wgpu::Buffer {
    device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }
    )
}

fn build_mesh(models: &Vec<Model>, vertices: &Vec<Vertex>, faces: &Vec<Face2>, edges: &Vec<Edge2>, ledges: &Vec<i32>, surfaces: &Vec<Surface>, textures: &Vec<Material>, mip_texs: &Vec<MipTex>) -> (Vec<RVertex>, Vec<u32>) {

    //https://github.com/fluffels/kwark/blob/master/src/Mesh.cpp
    let mut all_verts: Vec<Vec<RVertex>> = vec![Vec::new(); textures.len()];
    let mut render_offsets: Vec<u32> = Vec::new();
    let mut render_verts: Vec<RVertex> = Vec::new();
    let mut tracker = 0;
    for model in models {

        if tracker == 519 {
            println!("YOOO");
            continue;
        }
        tracker += 1;

        let first_face = model.face_id;
        let last_face = first_face + model.face_num;

        for j in first_face..last_face {

            let face = faces[j as usize];

            let tex_info = surfaces[face.tex_info as usize];
            let tex_id = tex_info.texture_id;
            let tex_type = textures[tex_id as usize].tex_type;
            let mip_tex = mip_texs[tex_info.texture_id as usize];
            if tex_type == TEX_DEBUG {
                continue;
            }

            let ledge_base = face.first_edge;

            let mut face_coords: Vec<Vertex> = Vec::new();
            
            for l in 0..face.num_edges {

                let edge_list_id = ledge_base + (l as i32);
                let edge_id = ledges[edge_list_id as usize];
                let edge = edges[edge_id.abs() as usize];
                let vert0 = vertices[edge.v[0] as usize];
                let vert1 = vertices[edge.v[1] as usize];
                if edge_id < 0 {
                    face_coords.push(vert1);
                    face_coords.push(vert0);
                }
                else {
                    face_coords.push(vert0);
                    face_coords.push(vert1);
                }
            }

            let r: f32 = rand::random();
            let g: f32 = rand::random();
            let b: f32 = rand::random();

            let (u0, v0) = calc_uv(face_coords[0], &tex_info);

            let rv0 = face_coords[0].to_rvertex(r, g, b, u0 / (mip_tex.width as f32), v0 / (mip_tex.height as f32), 0.0, 0.0, face.styles, [0.0, 0.0], -1);

            let marker = all_verts[tex_id as usize].len();

            //println!("{} {} {} {}", face.typelight, face.baselight, face.light[0], face.light[0]);

            for l in 1..face.num_edges {

                all_verts[tex_id as usize].push(rv0);

                let (u1, v1) = calc_uv(face_coords[(l as usize) * 2], &tex_info);
                let rv1 = face_coords[(l as usize) * 2].to_rvertex(r, g, b, u1 / (mip_tex.width as f32), v1 / (mip_tex.height as f32), 0.0, 0.0, face.styles, [0.0, 0.0], -1);
                all_verts[tex_id as usize].push(rv1);

                let (u2, v2) = calc_uv(face_coords[(l as usize) * 2 + 1], &tex_info);
                let rv2 = face_coords[(l as usize) * 2 + 1].to_rvertex(r, g, b, u2 / (mip_tex.width as f32), v2 / (mip_tex.height as f32), 0.0, 0.0, face.styles, [0.0, 0.0], -1);
                all_verts[tex_id as usize].push(rv2);
            } 

            if face.light_ofs != -1 {
                let mut uv_min: [f32; 2] = [u0, v0];
                let mut uv_max: [f32; 2] = [u0, v0];
                for l in marker..(all_verts[tex_id as usize].len()) {
                    let uv: [f32; 2] = [all_verts[tex_id as usize][l].u * mip_tex.width as f32, all_verts[tex_id as usize][l].v * mip_tex.height as f32];
                    if uv[0] < uv_min[0] {
                        uv_min[0] = uv[0];
                    }
                    if uv[1] < uv_min[1] {
                        uv_min[1] = uv[1];
                    }
                    if uv[0] > uv_max[0] {
                        uv_max[0] = uv[0];
                    }
                    if uv[1] > uv_max[1] {
                        uv_max[1] = uv[1];
                    }
                }

                uv_min[0] = (uv_min[0] / 16.0).floor();
                uv_min[1] = (uv_min[1] / 16.0).floor();
                uv_max[0] = (uv_max[0] / 16.0).ceil();
                uv_max[1] = (uv_max[1] / 16.0).ceil();

                let extent: [f32; 2] = [(uv_max[0] - uv_min[0]).floor() + 1.0, (uv_max[1] - uv_min[1]).floor() + 1.0];
            
                for l in marker..(all_verts[tex_id as usize].len()) {
                    let uv: [f32; 2] = [all_verts[tex_id as usize][l].u * mip_tex.width as f32, all_verts[tex_id as usize][l].v * mip_tex.height as f32];
                    all_verts[tex_id as usize][l].lu = (uv[0] / 16.0) - uv_min[0];
                    all_verts[tex_id as usize][l].lv = (uv[1] / 16.0) - uv_min[1];
                    all_verts[tex_id as usize][l].light_id = face.light_ofs;
                    all_verts[tex_id as usize][l].extent = extent;
                }
            }
        }
    }

    for i in 0..all_verts.len() {
        for j in 0..all_verts[i].len() {
            render_verts.push(all_verts[i][j]);
        }
        render_offsets.push(all_verts[i].len() as u32);
    }

    (render_verts, render_offsets)
}

fn calc_uv(vert: Vertex, tex_info: &Surface) -> (f32, f32) {
    let u = cgmath::dot(vert.get_cgvec3(), to_vec3(tex_info.vector_s)) + tex_info.dist_s;
    let v = cgmath::dot(vert.get_cgvec3(), to_vec3(tex_info.vector_t)) + tex_info.dist_t;
    (u, v)
}

fn extract_entities(entities: &str) -> (Vec<EntityInfo>, Vec<i32>) {
    
    let ents_start: Vec<_> = entities.match_indices("{").collect();
    let ents_end: Vec<_> = entities.match_indices("}").collect();

    let mut entity_info: Vec<EntityInfo> = Vec::new();
    let mut collidable: Vec<i32> = Vec::new();
    collidable.push(0); //Push world to collidable list first
    for i in 0..ents_start.len() {
        let ent_info = entities[(ents_start[i].0 + 1)..(ents_end[i].0)].trim();
        //println!("{}\n\n", ent_info);
        let words: Vec<&str> = ent_info.split_whitespace().collect();
        let mut model_num = -1;
        let mut collide = 1; //Default to physent (1) (-1 for no player movement clip)
        for j in (0..words.len()).step_by(2) {
            if j + 1 >= words.len() {
                break;
            }
            if words[j + 1].find("trigger").is_some() {
                collide = -1;
            }
            if words[j].find("model").is_some() {
                let nums: Vec<&str> = words[j + 1].matches(char::is_numeric).collect();
                let mut num_s: String = "".to_string();
                for num in nums {
                    num_s += num;
                }
                model_num = num_s.parse::<i32>().unwrap();
            }
        }
        /*if collide == 1 {
            println!("{}\n\n", ent_info);
        }*/
        
        if collide != -1 && model_num != -1 {
            collidable.push(model_num);
        }
        entity_info.push(EntityInfo { type_e: model_num, render_debug: -1, collide })
    }

    (entity_info, collidable)
}

pub fn to_vec3(inp: [f32; 3]) -> cgmath::Vector3<f32> {
    cgmath::Vector3::new(inp[0], inp[1], inp[2])
}

pub fn read_rgb_lights(map_name: &str) -> Vec<u8> {

    let light_dir = "./res/".to_string() + map_name + ".lit";
    let light_file = std::fs::read(light_dir);

    let mut light_rgba: Vec<u8> = Vec::new();

    let light_file = match light_file {
        Ok(l_bytes) => {
            if l_bytes[0] == 'Q' as u8 && l_bytes[1] == 'L' as u8 && l_bytes[2] == 'I' as u8 && l_bytes[3] == 'T' as u8 {
                let version = bytemuck::from_bytes::<u32>(&l_bytes[4..8]);
                for i in (8..l_bytes.len()).step_by(3) {
                    light_rgba.push(l_bytes[i]);
                    light_rgba.push(l_bytes[i + 1]);
                    light_rgba.push(l_bytes[i + 2]);
                    light_rgba.push(255);
                }
            }
        }
        Err(error) => {
            //No rgb lighting
        }
    };

    light_rgba
}

pub fn to_bsp2(nodes: &Vec<Node>, clip_nodes: &Vec<ClipNode>, edges: &Vec<Edge>, faces: &Vec<Face>, leafs: &Vec<Leaf>) -> (Vec<Node2>, Vec<ClipNode2>, Vec<Edge2>, Vec<Face2>, Vec<Leaf2>) {

    let mut nodes_2 = Vec::new();
    let mut clip_nodes_2 = Vec::new();
    let mut edges_2 = Vec::new();
    let mut faces_2 = Vec::new();
    let mut leafs_2 = Vec::new();

    for i in 0..nodes.len() {
        nodes_2.push(Node2 { plane_num: nodes[i].plane_id, children: [nodes[i].front as i32, nodes[i].back as i32], mins: [nodes[i].bboxs.min[0] as u16, nodes[i].bboxs.min[1] as u16, nodes[i].bboxs.min[2] as u16], maxs: [nodes[i].bboxs.max[0] as u16, nodes[i].bboxs.max[1] as u16, nodes[i].bboxs.max[2] as u16],
        first_face: nodes[i].face_id as u32, num_faces: nodes[i].face_num as u32 });
    }

    for i in 0..clip_nodes.len() {
        clip_nodes_2.push(ClipNode2 { plane_num: clip_nodes[i].plane_num as i32, children: [clip_nodes[i].front as i32, clip_nodes[i].back as i32] });
    }

    for i in 0..edges.len() {
        edges_2.push(Edge2 { v: [edges[i].vertex0 as u32, edges[i].vertex1 as u32] });
    }

    for i in 0..faces.len() {
        faces_2.push(Face2 { plane_num: faces[i].plane_id as i32, side: faces[i].side as i32, first_edge: faces[i].ledge_id, num_edges: faces[i].ledge_num as i32, tex_info: faces[i].texinfo_id as i32, styles: [faces[i].typelight, faces[i].baselight, faces[i].light[0], faces[i].light[1]], light_ofs: faces[i].lightmap });
    }

    for i in 0..leafs.len() {
        leafs_2.push(Leaf2 { contents: leafs[i].ltype, visofs: leafs[i].vislist, mins: [leafs[i].bound.min[0] as u16, leafs[i].bound.min[1] as u16, leafs[i].bound.min[2] as u16], maxs: [leafs[i].bound.max[0] as u16, leafs[i].bound.max[1] as u16, leafs[i].bound.max[2] as u16],
        first_mark_surface: leafs[i].lface_id as u32, num_mark_surfaces: leafs[i].lface_num as u32, ambient_level: [leafs[i].sndwater, leafs[i].sndsky, leafs[i].sndslime, leafs[i].sndlava] });
    }

    return (nodes_2, clip_nodes_2, edges_2, faces_2, leafs_2);
}

pub fn get_hull0(clip_len: usize) -> Hull {

    Hull { clip_nodes_id: 0, planes_id: 0, first_clip_node: 0, last_clip_node: clip_len as i32 - 1, mins: [0.0, 0.0, 0.0], maxs: [0.0, 0.0, 0.0] }
}

pub fn get_hulls(hulls: &mut Vec<Hull>, clip_nodes_id: usize, planes_id: usize, clip_len: usize) {

    hulls.push(Hull { clip_nodes_id, planes_id, first_clip_node: 0, last_clip_node: clip_len as i32 - 1, mins: [-16.0, -16.0, -24.0], maxs: [16.0, 16.0, 32.0] });
    hulls.push(Hull { clip_nodes_id, planes_id, first_clip_node: 0, last_clip_node: clip_len as i32 - 1, mins: [-32.0, -32.0, -24.0], maxs: [32.0, 32.0, 64.0] });
}
*/