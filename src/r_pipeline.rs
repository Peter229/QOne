/*use crate::texture;
use crate::uniform;

pub fn render_pipeline(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor, vs_module: wgpu::ShaderModuleDescriptor, fs_module: wgpu::ShaderModuleDescriptor, vertex_format: wgpu::VertexBufferLayout) -> wgpu::RenderPipeline {

    let vs_shader = device.create_shader_module(&vs_module);
    let fs_shader = device.create_shader_module(&fs_module);

    let uniform_bind_group_layout = uniform::Uniforms::get_bind_group_layout(device);

    let texture_bind_group_layout = texture::Texture::get_bind_group_layout(device);

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vs_shader,
            entry_point: "main",
            buffers: &[vertex_format],
        },
        fragment: Some(wgpu::FragmentState {
            module: &fs_shader,
            entry_point: "main",
            targets: &[sc_desc.format.into()],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::Front,
            polygon_mode: wgpu::PolygonMode::Fill,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: texture::Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState {
                constant: 0,
                slope_scale: 0.0,
                clamp: 0.0,
            },
            clamp_depth: false,
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
    });

    render_pipeline
}*/

use crate::texture;

pub fn pipeline(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor, render_pipeline_layout: &wgpu::PipelineLayout, vs_module_s: wgpu::ShaderModuleSource, fs_module_s: wgpu::ShaderModuleSource, vertex_format: wgpu::VertexBufferDescriptor, primitive: wgpu::PrimitiveTopology) -> wgpu::RenderPipeline {
    
    let vs_module = device.create_shader_module(vs_module_s);
    let fs_module = device.create_shader_module(fs_module_s);

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(render_pipeline_layout),
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        }),
        rasterization_state: Some(
            wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Front,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }
        ),
        color_states: &[
            wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE, //color_blend: wgpu::BlendDescriptor::REPLACE, //alpha_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            },
        ],
        primitive_topology: primitive, //LineList TriangleList
        depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
            format: texture::Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilStateDescriptor::default(),
        }),
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint32,
            vertex_buffers: &[vertex_format],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });
    render_pipeline
}

/*
color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add
                    },
                    */