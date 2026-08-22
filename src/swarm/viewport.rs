//-- swarm/viewport.rs ---------------------------------------------------------------------------------------------------------------
use	std::sync::Arc;
use	wgpu::{
    Device, Queue, RenderPipeline, Buffer, BindGroup,
    BufferUsages, TextureFormat, PrimitiveTopology,
    PipelineLayoutDescriptor, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    ShaderStages, BindingType, BufferBindingType, BindGroupDescriptor,
    BindGroupEntry, ShaderModuleDescriptor, ShaderSource, VertexState,
    FragmentState, ColorTargetState, ColorWrites, PrimitiveState,
    MultisampleState, VertexBufferLayout, VertexStepMode, VertexAttribute,
    VertexFormat, RenderPipelineDescriptor, util::DeviceExt,
};
use	crate::silo::U32;
use	crate::silo::cast::ISliceExt;
use	crate::swarm::scene::Camera;
/// Represents a 3D OBJ render mode in the native viewport.
#[derive( Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjRenderMode
{
    Points,
    Wireframe,
    Facets,
    ShadedWire,
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Uniform struct passed to viewport shaders.
#[repr( C)]
#[derive( Clone, Copy)]
pub struct ViewportUniforms
{
    pub _MatViewProj: [f32; 16],
    pub _Color:       [f32; 4],
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Vertex attribute layout for 3D position + normal.
#[repr( C)]
#[derive( Clone, Copy, Debug)]
pub struct ViewportVertex
{
    pub _Pos:    [f32; 3],
    pub _Normal: [f32; 3],
}

impl ViewportVertex
{
    pub fn	Layout<'a>() -> VertexBufferLayout<'a>
    {
        VertexBufferLayout {
            array_stride: std::mem::size_of::< ViewportVertex>() as wgpu::BufferAddress,
            step_mode:    VertexStepMode::Vertex,
            attributes:   &[
                VertexAttribute {
                    offset:          0,
                    shader_location: 0,
                    format:          VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset:          12,
                    shader_location: 1,
                    format:          VertexFormat::Float32x3,
                },
            ],
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Hardware GPU Mesh resource allocated in VRAM.
#[derive( Clone)]
pub struct GpuMesh
{
    pub _VertexBuffer:      Arc< Buffer>,
    pub _IndexBuffer:       Option< Arc< Buffer>>,
    pub _WireIndexBuffer:   Option< Arc< Buffer>>,
    pub _VertexCount:       U32,
    pub _IndexCount:        U32,
    pub _WireIndexCount:    U32,
    pub _BboxMin:           [f32; 3],
    pub _BboxMax:           [f32; 3],
}

impl GpuMesh
{
    /// Creates a GPU mesh from raw vertex positions and triangles.
    pub fn	FromVerticesAndIndices(
        device: &Device,
        vertices: &[ViewportVertex],
        indices: Option< &[u32]>,
        wireIndices: Option< &[u32]>,
        bboxMin: [f32; 3],
        bboxMax: [f32; 3],
    ) -> Self
    {
        let  	vBuf = device.create_buffer_init( &wgpu::util::BufferInitDescriptor {
            label: Some( "ViewportVertexBuffer"),
            contents: vertices.CastSlice(),
            usage: BufferUsages::VERTEX,
        });

        let  	iBuf = indices.map( |idx| {
            Arc::new( device.create_buffer_init( &wgpu::util::BufferInitDescriptor {
                label: Some( "ViewportIndexBuffer"),
                contents: idx.CastSlice(),
                usage: BufferUsages::INDEX,
            }))
        });

        let  	wBuf = wireIndices.map( |widx| {
            Arc::new( device.create_buffer_init( &wgpu::util::BufferInitDescriptor {
                label: Some( "ViewportWireIndexBuffer"),
                contents: widx.CastSlice(),
                usage: BufferUsages::INDEX,
            }))
        });

        GpuMesh {
            _VertexBuffer:    Arc::new( vBuf),
            _IndexBuffer:     iBuf,
            _WireIndexBuffer: wBuf,
            _VertexCount:     U32( vertices.len() as u32),
            _IndexCount:      U32( indices.map( |i| i.len() as u32).unwrap_or( 0)),
            _WireIndexCount:  U32( wireIndices.map( |w| w.len() as u32).unwrap_or( 0)),
            _BboxMin:         bboxMin,
            _BboxMax:         bboxMax,
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

const VIEWPORT_WGSL: &str = r#"
struct Uniforms {
    mat_view_proj: mat4x4<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.mat_view_proj * vec4<f32>(model.position, 1.0);
    out.normal = model.normal;
    return out;
}

@fragment
fn fs_shaded(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.5, 0.8, 1.0));
    let n = normalize(in.normal);
    let diff = max(dot(n, light_dir), 0.3);
    return vec4<f32>(uniforms.color.rgb * diff, uniforms.color.a);
}

@fragment
fn fs_flat(in: VertexOutput) -> @location(0) vec4<f32> {
    return uniforms.color;
}
"#;

// ---------------------------------------------------------------------------------------------------------------------------------

/// Hardware 3D viewport renderer supporting Points, Wireframe, Facets, and ShadedWire.
pub struct ViewportRenderer
{
    _Device:            Arc< Device>,
    _Queue:             Arc< Queue>,
    _PointsPipeline:    RenderPipeline,
    _WireframePipeline: RenderPipeline,
    _FacetsPipeline:    RenderPipeline,
    _UniformBuffer:     Buffer,
    _BindGroup:         BindGroup,
}

impl ViewportRenderer
{
    pub fn	New( device: Arc< Device>, queue: Arc< Queue>, targetFormat: TextureFormat) -> Self
    {
        let  	shader = device.create_shader_module( ShaderModuleDescriptor {
            label: Some( "ViewportShader"),
            source: ShaderSource::Wgsl( VIEWPORT_WGSL.into()),
        });

        let  	uniformSize = std::mem::size_of::< ViewportUniforms>() as wgpu::BufferAddress;
        let  	uniformBuffer = device.create_buffer( &wgpu::BufferDescriptor {
            label: Some( "ViewportUniformBuffer"),
            size: uniformSize,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let  	bindGroupLayout = device.create_bind_group_layout( &BindGroupLayoutDescriptor {
            label: Some( "ViewportBindGroupLayout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let  	bindGroup = device.create_bind_group( &BindGroupDescriptor {
            label: Some( "ViewportBindGroup"),
            layout: &bindGroupLayout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniformBuffer.as_entire_binding(),
                },
            ],
        });

        let  	pipelineLayout = device.create_pipeline_layout( &PipelineLayoutDescriptor {
            label: Some( "ViewportPipelineLayout"),
            bind_group_layouts: &[ Some( &bindGroupLayout) ],
            immediate_size: 0,
        });

        // 1. Points Pipeline
        let  	pointsPipeline = device.create_render_pipeline( &RenderPipelineDescriptor {
            label: Some( "ViewportPointsPipeline"),
            layout: Some( &pipelineLayout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some( "vs_main"),
                buffers: &[ Some( ViewportVertex::Layout()) ],
                compilation_options: Default::default(),
            },
            fragment: Some( FragmentState {
                module: &shader,
                entry_point: Some( "fs_flat"),
                targets: &[ Some( ColorTargetState {
                    format: targetFormat,
                    blend: Some( wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }) ],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::PointList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // 2. Wireframe Pipeline
        let  	wireframePipeline = device.create_render_pipeline( &RenderPipelineDescriptor {
            label: Some( "ViewportWireframePipeline"),
            layout: Some( &pipelineLayout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some( "vs_main"),
                buffers: &[ Some( ViewportVertex::Layout()) ],
                compilation_options: Default::default(),
            },
            fragment: Some( FragmentState {
                module: &shader,
                entry_point: Some( "fs_flat"),
                targets: &[ Some( ColorTargetState {
                    format: targetFormat,
                    blend: Some( wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }) ],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::LineList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // 3. Facets / Shaded Pipeline
        let  	facetsPipeline = device.create_render_pipeline( &RenderPipelineDescriptor {
            label: Some( "ViewportFacetsPipeline"),
            layout: Some( &pipelineLayout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some( "vs_main"),
                buffers: &[ Some( ViewportVertex::Layout()) ],
                compilation_options: Default::default(),
            },
            fragment: Some( FragmentState {
                module: &shader,
                entry_point: Some( "fs_shaded"),
                targets: &[ Some( ColorTargetState {
                    format: targetFormat,
                    blend: Some( wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }) ],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        ViewportRenderer {
            _Device:            device,
            _Queue:             queue,
            _PointsPipeline:    pointsPipeline,
            _WireframePipeline: wireframePipeline,
            _FacetsPipeline:    facetsPipeline,
            _UniformBuffer:     uniformBuffer,
            _BindGroup:         bindGroup,
        }
    }

    /// Updates uniform buffer with camera transform and model view matrix.
    pub fn	UpdateUniforms( &self, camera: &Camera, width: f32, height: f32, color: [f32; 4])
    {
        let  	matProj = camera.CalcViewProjMatrix( width, height);
        let  	uniforms = ViewportUniforms {
            _MatViewProj: matProj,
            _Color:       color,
        };
        let  	uSlice: &[ViewportUniforms] = std::slice::from_ref( &uniforms);
        self._Queue.write_buffer( &self._UniformBuffer, 0, uSlice.CastSlice());
    }

    /// Renders a GPU mesh using the selected ObjRenderMode (Points, Wireframe, Facets, ShadedWire).
    pub fn	RenderMesh(
        &self,
        renderPass: &mut wgpu::RenderPass,
        mesh: &GpuMesh,
        mode: ObjRenderMode,
    )
    {
        renderPass.set_bind_group( 0, &self._BindGroup, &[]);
        renderPass.set_vertex_buffer( 0, mesh._VertexBuffer.slice( ..));

        match mode {
            ObjRenderMode::Points => {
                renderPass.set_pipeline( &self._PointsPipeline);
                renderPass.draw( 0..mesh._VertexCount.AsU32(), 0..1);
            }
            ObjRenderMode::Wireframe => {
                renderPass.set_pipeline( &self._WireframePipeline);
                if let Some( ref wIdx) = mesh._WireIndexBuffer {
                    renderPass.set_index_buffer( wIdx.slice( ..), wgpu::IndexFormat::Uint32);
                    renderPass.draw_indexed( 0..mesh._WireIndexCount.AsU32(), 0, 0..1);
                } else if let Some( ref idx) = mesh._IndexBuffer {
                    renderPass.set_index_buffer( idx.slice( ..), wgpu::IndexFormat::Uint32);
                    renderPass.draw_indexed( 0..mesh._IndexCount.AsU32(), 0, 0..1);
                } else {
                    renderPass.draw( 0..mesh._VertexCount.AsU32(), 0..1);
                }
            }
            ObjRenderMode::Facets => {
                renderPass.set_pipeline( &self._FacetsPipeline);
                if let Some( ref idx) = mesh._IndexBuffer {
                    renderPass.set_index_buffer( idx.slice( ..), wgpu::IndexFormat::Uint32);
                    renderPass.draw_indexed( 0..mesh._IndexCount.AsU32(), 0, 0..1);
                } else {
                    renderPass.draw( 0..mesh._VertexCount.AsU32(), 0..1);
                }
            }
            ObjRenderMode::ShadedWire => {
                // 1. Draw shaded facets
                renderPass.set_pipeline( &self._FacetsPipeline);
                if let Some( ref idx) = mesh._IndexBuffer {
                    renderPass.set_index_buffer( idx.slice( ..), wgpu::IndexFormat::Uint32);
                    renderPass.draw_indexed( 0..mesh._IndexCount.AsU32(), 0, 0..1);
                } else {
                    renderPass.draw( 0..mesh._VertexCount.AsU32(), 0..1);
                }

                // 2. Draw wireframe overlay on top
                renderPass.set_pipeline( &self._WireframePipeline);
                if let Some( ref wIdx) = mesh._WireIndexBuffer {
                    renderPass.set_index_buffer( wIdx.slice( ..), wgpu::IndexFormat::Uint32);
                    renderPass.draw_indexed( 0..mesh._WireIndexCount.AsU32(), 0, 0..1);
                }
            }
        }
    }
}
