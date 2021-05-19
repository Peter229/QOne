use std::io;
use std::io::prelude::*;
use std::fs::File;
use wgpu::util::DeviceExt;
use std::io::{stdin,stdout,Write};

use crate::qone_texture;
use crate::qone_types::*;

//https://www.gamers.org/dEngine/quake/spec/quake-spec34/qkspec_4.htm

const CONTENTS_SOLID: i32 = -2;
const CONTENTS_CLIP: i32 = -8;
const CONTENTS_EMPTY: i32 = -1;
const DIST_EPSILON: f32 = 0.03125;

pub struct Bsp {
    pub vertex_buffer: wgpu::Buffer,
    pub num_verts: u32,
    models: Vec<Model>,
    nodes: Vec<Node>,
    leafs: Vec<Leaf>,
    planes: Vec<Plane>,
    clip_nodes: Vec<ClipNode>,
    pub textures: Vec<Material>,
    pub render_offsets: Vec<u32>,
    pub entity_info: Vec<EntityInfo>,
    pub collidable: Vec<i32>,
}

//https://discord.com/channels/380484403458998276/382248269444808724/835817766228721664
impl Bsp {

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, layout: &wgpu::BindGroupLayout, light_layout: &wgpu::BindGroupLayout) -> Bsp {

        let mut dir = "./res/e1m1.bsp".to_string();
        let mut bytes = std::fs::read(dir).unwrap();

        //Load header
        let header = *bytemuck::from_bytes::<Header>(&bytes[..std::mem::size_of::<Header>()]);

        //Load data
        //Render data
        let vertices = parse_data::<Vertex>(header.vertices, &bytes);
        let faces = parse_data::<Face>(header.faces, &bytes);
        let edges = parse_data::<Edge>(header.edges, &bytes);
        let models = parse_data::<Model>(header.models, &bytes);
        let ledges = parse_data::<i32>(header.ledges, &bytes);
        let light_maps = parse_data::<u8>(header.lightmaps, &bytes);

        //Textures
        let pallete = qone_texture::load_pallete();
        let (textures, mip_texs) = qone_texture::parse_textures(header.miptex, &bytes, &pallete, device, queue, layout);
        let surfaces = parse_data::<Surface>(header.texinfo, &bytes);

        //Collision data
        let nodes = parse_data::<Node>(header.nodes, &bytes);
        let leafs = parse_data::<Leaf>(header.leaves, &bytes);
        let planes = parse_data::<Plane>(header.planes, &bytes);
        let clip_nodes = parse_data::<ClipNode>(header.clipnodes, &bytes);
    
        //Entities
        let entities = parse_data::<u8>(header.entities, &bytes);
        let (entity_info, collidable) = extract_entities(std::str::from_utf8(&entities).unwrap());

        //Build mesh
        let (render_verts, render_offsets) = build_mesh(&models, &vertices, &faces, &edges, &ledges, &surfaces, &textures, &mip_texs);

        //Generate buffers
        let vertex_buffer = generate_buffer(device, &render_verts);

        let num_verts = render_verts.len() as u32;

        Bsp { vertex_buffer, num_verts, models, nodes, leafs, planes, clip_nodes, textures, render_offsets, entity_info, collidable } 
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
            return self.recusive_hull_check(id, node.front as i32, p1f, p2f, p1, p2, trace);
        }
        if t1 < 0.0 && t2 < 0.0 {
            return self.recusive_hull_check(id, node.back as i32, p1f, p2f, p1, p2, trace);
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
        let mut side_n = node.front as i32;
        if side {
            side_n = node.back as i32;
        }

        if (!self.recusive_hull_check(id, side_n, p1f, midf, p1, mid, trace)) {
            return false;
        }

        let side_t = (t1 < 0.0);
        let mut side_tn = node.back as i32;
        if side_t {
            side_tn = node.front as i32;
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

    fn hull_point_contents(&mut self, id: i32, mut num: i32, p: cgmath::Vector3<f32>) -> i32 {

        while num >= 0 {
            if num < self.models[id as usize].node_id1 || num > self.models[id as usize].node_id2 {
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
                num = node.back as i32;
            }
            else {
                num = node.front as i32;
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
            usage: wgpu::BufferUsage::VERTEX,
        }
    )
}

fn build_mesh(models: &Vec<Model>, vertices: &Vec<Vertex>, faces: &Vec<Face>, edges: &Vec<Edge>, ledges: &Vec<i32>, surfaces: &Vec<Surface>, textures: &Vec<Material>, mip_texs: &Vec<MipTex>) -> (Vec<RVertex>, Vec<u32>) {

    //https://github.com/fluffels/kwark/blob/master/src/Mesh.cpp
    let mut all_verts: Vec<Vec<RVertex>> = vec![Vec::new(); textures.len()];
    let mut render_offsets: Vec<u32> = Vec::new();
    let mut render_verts: Vec<RVertex> = Vec::new();
    for model in models {
        let first_face = model.face_id;
        let last_face = first_face + model.face_num;

        for j in first_face..last_face {

            let face = faces[j as usize];

            let tex_info = surfaces[face.texinfo_id as usize];
            let tex_id = tex_info.texture_id;
            let tex_type = textures[tex_id as usize].tex_type;
            let mip_tex = mip_texs[tex_info.texture_id as usize];
            if tex_type == TEX_DEBUG {
                continue;
            }

            let ledge_base = face.ledge_id;

            let mut face_coords: Vec<Vertex> = Vec::new();
            
            for l in 0..face.ledge_num {

                let edge_list_id = ledge_base + (l as i32);
                let edge_id = ledges[edge_list_id as usize];
                let edge = edges[edge_id.abs() as usize];
                let vert0 = vertices[edge.vertex0 as usize];
                let vert1 = vertices[edge.vertex1 as usize];
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

            let rv0 = face_coords[0].to_rvertex(r, g, b, u0 / (mip_tex.width as f32), v0 / (mip_tex.height as f32), 0.0, 0.0);
                
            for l in 1..face.ledge_num {

                all_verts[tex_id as usize].push(rv0);

                let (u1, v1) = calc_uv(face_coords[(l as usize) * 2], &tex_info);
                let rv1 = face_coords[(l as usize) * 2].to_rvertex(r, g, b, u1 / (mip_tex.width as f32), v1 / (mip_tex.height as f32), 0.0, 0.0);
                all_verts[tex_id as usize].push(rv1);

                let (u2, v2) = calc_uv(face_coords[(l as usize) * 2 + 1], &tex_info);
                let rv2 = face_coords[(l as usize) * 2 + 1].to_rvertex(r, g, b, u2 / (mip_tex.width as f32), v2 / (mip_tex.height as f32), 0.0, 0.0);
                all_verts[tex_id as usize].push(rv2);
            }

            /*if face.lightmap != -1 {
                let mut uv_min: [f32; 2] = [u0, v0];
                let mut uv_max: [f32; 2] = [u0, v0];
                for i in 0..face_coords.len() {
                    let uv: [f32; 2] = [face_coords.u
                }
            }*/
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