use crate::texture;

pub const TEX_DEBUG: i32 = 0;
pub const TEX_SKY: i32 = 1;
pub const TEX_FLUID: i32 = 2;
pub const TEX_DEFAULT: i32 = 3;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RVertex {
    x: f32,
    y: f32,
    z: f32,
    r: f32,
    b: f32,
    g: f32,
    u: f32,
    v: f32,
    lu: f32,
    lv: f32,
}

impl RVertex {

    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<RVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float2,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Face {
    pub plane_id: u16,
    pub side: u16,
    pub ledge_id: i32,
    pub ledge_num: u16,
    pub texinfo_id: u16,
    pub typelight: u8,
    pub baselight: u8,
    pub light: [u8; 2],
    pub lightmap: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Edge {
    pub vertex0: u16,
    pub vertex1: u16,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vertex {
    pub fn to_rvertex(&self, r: f32, g: f32, b: f32, u: f32, v: f32, lu: f32, lv: f32) -> RVertex {
        RVertex { x: self.x, y: self.y, z: self.z, r, g, b, u, v, lu, lv }
    }

    pub fn get_cgvec3(&self) -> cgmath::Vector3<f32> {
        cgmath::Vector3::new(self.x, self.y, self.z)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BoundBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Model {
    pub bound: BoundBox,
    pub origin: [f32; 3],
    pub node_id0: i32,
    pub node_id1: i32,
    pub node_id2: i32,
    pub node_id3: i32,
    pub num_leafs: i32,
    pub face_id: i32,
    pub face_num: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BoundBoxShort {
    pub min: [i16; 3],
    pub max: [i16; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Node {
    pub plane_id: i32,
    pub front: u16,
    pub back: u16,
    pub bboxs: BoundBoxShort,
    pub face_id: u16,
    pub face_num: u16,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Leaf {
    pub ltype: i32,
    pub vislist: i32,
    pub bound: BoundBoxShort,
    pub lface_id: u16,
    pub lface_num: u16,
    pub sndwater: u8,
    pub sndsky: u8,
    pub sndslime: u8,
    pub sndlava: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Plane {
    pub normal: [f32; 3],
    pub dist: f32,
    pub ptype: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ClipNode {
    pub plane_num: u32,
    pub front: i16,
    pub back: i16,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Surface {
    pub vector_s: [f32; 3],
    pub dist_s: f32,
    pub vector_t: [f32; 3],
    pub dist_t: f32,
    pub texture_id: u32,
    pub animated: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MipTex {
    pub name: [u8; 16],
    pub width: u32,
    pub height: u32,
    pub offset_1: u32,
    pub offset_2: u32,
    pub offset_4: u32,
    pub offset_8: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Header {
    pub version: i32,
    pub entities: Entry,
    pub planes: Entry,
    pub miptex: Entry,
    pub vertices: Entry,
    pub visilist: Entry,
    pub nodes: Entry,
    pub texinfo: Entry,
    pub faces: Entry,
    pub lightmaps: Entry,
    pub clipnodes: Entry,
    pub leaves: Entry,
    pub lface: Entry,
    pub edges: Entry,
    pub ledges: Entry,
    pub models: Entry,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Entry {
    pub offset: i32,
    pub size: i32,
}

pub struct Trace {
    pub all_solid: bool,
    pub in_open: bool,
    pub in_water: bool,
    pub starts_solid: bool,
    pub plane: Plane,
    pub fraction: f32,
    pub end_pos: cgmath::Vector3<f32>,
    pub ent: i32,
}

impl Trace {
    pub fn new(end_pos: cgmath::Vector3<f32>) -> Trace {
        Trace { all_solid: false, in_open: false, in_water: false, starts_solid: false, 
                plane: Plane { normal: [0.0, 0.0, 0.0], dist: 0.0, ptype: 0 }, fraction: 1.0, 
            end_pos, ent: -1 }
    }
}

pub struct Material {
    pub diffuse_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
    pub tex_type: i32,
}

pub struct EntityInfo {
    pub type_e: i32,
    pub render_debug: i32,
    pub collide: i32,
}